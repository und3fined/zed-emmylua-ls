use std::env::consts::ARCH;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use zed::settings::LspSettings;
use zed_extension_api::{
  self as zed, LanguageServerId, Result, Worktree,
  serde_json::{self, Value},
};

struct EmmyLuaExtension;

impl EmmyLuaExtension {
  fn binary_exists(&self, path: &PathBuf) -> bool {
    std::fs::metadata(path).is_ok_and(|stat| stat.is_file())
  }

  fn get_binary_name(&self) -> &'static str {
    let (platform, _) = zed::current_platform();
    match platform {
      zed::Os::Windows => "emmylua_ls.exe",
      _ => "emmylua_ls",
    }
  }

  fn assets_pattern(&self) -> Result<String, String> {
    let (platform, arch) = zed::current_platform();

    let (platform_str, arch_str, extension) = match (platform, arch) {
      (zed::Os::Mac, zed::Architecture::Aarch64) => ("darwin", "arm64", "tar.gz"),
      (zed::Os::Mac, zed::Architecture::X8664) => ("darwin", "x64", "tar.gz"),
      (zed::Os::Linux, zed::Architecture::Aarch64) => ("linux", "aarch64-glibc.2.17", "tar.gz"),
      (zed::Os::Linux, zed::Architecture::X8664) => ("linux", "x64-glibc.2.17", "tar.gz"),
      (zed::Os::Windows, zed::Architecture::Aarch64) => ("win32", "arm64", "zip"),
      (zed::Os::Windows, zed::Architecture::X8664) => ("win32", "x64", "zip"),
      _ => {
        return Err(format!(
          "unsupported platform/architecture: {platform:?}/{arch:?}"
        ));
      }
    };

    Ok(format!(
      "{platform}-{arch}.{extension}",
      platform = platform_str,
      arch = arch_str,
      extension = extension
    ))
  }

  fn check_and_install_server(&mut self, language_server_id: &LanguageServerId) -> Result<PathBuf> {
    let emmylua_update_lock = PathBuf::from("./tmp/emmylua_update.lock");
    let mut out_of_date = true;
    let mut current_version = "latest".to_string();
    let mut _last_checked = 0u64;

    // read emmylua_lock if it exists and check content to decide if we can update
    if emmylua_update_lock.exists() {
      if let Ok(content) = std::fs::read_to_string(&emmylua_update_lock) {
        let lock_info = content.split_once('\n').unwrap_or((content.as_str(), ""));

        current_version = lock_info.0.trim().to_string();
        _last_checked = if let Ok(ts) = lock_info.1.trim().parse::<u64>() {
          ts
        } else {
          0
        };

        let current_time = SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .as_secs();

        if current_time - _last_checked < 24 * 60 * 60 {
          out_of_date = false;
        }
      }
    }

    let binary_name = self.get_binary_name();
    let server_path = PathBuf::from("./bin").join(binary_name);

    if self.binary_exists(&server_path) && !out_of_date {
      return Ok(server_path);
    }

    zed::set_language_server_installation_status(
      language_server_id,
      &zed::LanguageServerInstallationStatus::CheckingForUpdate,
    );

    let release_result = zed::latest_github_release(
      "EmmyLuaLs/emmylua-analyzer-rust",
      zed::GithubReleaseOptions {
        require_assets: true,
        pre_release: false,
      },
    );

    if release_result.is_err() {
      if self.binary_exists(&server_path) {
        // If we can't reach GitHub but have a binary, just use it
        zed::set_language_server_installation_status(
          language_server_id,
          &zed::LanguageServerInstallationStatus::None,
        );
        return Ok(server_path);
      } else {
        return Err(format!(
          "Failed to fetch latest release info: {}",
          release_result.err().unwrap()
        ));
      }
    }

    let latest_release = release_result.unwrap();
    if latest_release.version == current_version && self.binary_exists(&server_path) {
      // Already up to date
      zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::None,
      );

      return Ok(server_path);
    }

    let assets_name = self.assets_pattern()?;
    let archive_name = format!("emmylua_ls-{}", assets_name);

    let download_url = latest_release
      .assets
      .iter()
      .find(|asset| asset.name == archive_name)
      .map(|asset| asset.download_url.clone());

    let archive_path = format!("./tmp/emmylua_ls-{}", latest_release.version);
    let (file_type, _extension) = if assets_name.ends_with(".zip") {
      (zed::DownloadedFileType::Zip, "zip")
    } else {
      (zed::DownloadedFileType::GzipTar, "tar.gz")
    };

    zed::set_language_server_installation_status(
      language_server_id,
      &zed::LanguageServerInstallationStatus::Downloading,
    );

    // Download the archive - this will extract to a directory without the extension
    zed::download_file(&download_url.unwrap().as_ref(), &archive_path, file_type)?;

    // Find the binary using recursive search
    let found_binary_path = self.find_binary_recursively("./tmp", binary_name)?;

    // If the binary is not in the expected location, copy it there
    if found_binary_path != server_path {
      std::fs::create_dir_all(server_path.parent().unwrap()).map_err(|e| e.to_string())?;
      std::fs::copy(&found_binary_path, &server_path).map_err(|e| e.to_string())?;
    }

    // Clean up the archive file
    let _ = std::fs::remove_dir_all(&archive_path);

    // write emmylua_lock with new version and current timestamp
    let current_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs();
    let lock_content = format!("{}\n{}", latest_release.version, current_time);
    std::fs::write(&emmylua_update_lock, lock_content).map_err(|e| e.to_string())?;

    zed::set_language_server_installation_status(
      language_server_id,
      &zed::LanguageServerInstallationStatus::None,
    );

    Ok(server_path)
  }

  fn find_binary_recursively(&self, dir: &str, binary_name: &str) -> Result<PathBuf, String> {
    let base_path = std::path::Path::new(dir);

    // First check common binary locations in order of preference
    let common_paths = vec![
      base_path.join(binary_name),
      base_path.join("bin").join(binary_name),
      base_path.join("emmylua_ls").join(binary_name),
      base_path.join("emmylua_ls").join("bin").join(binary_name),
    ];

    for path in &common_paths {
      if self.binary_exists(path) {
        return Ok(path.clone());
      }
    }

    // If not found in common locations, do recursive search
    fn search_directory(
      dir: &std::path::Path,
      binary_name: &str,
    ) -> Result<PathBuf, std::io::Error> {
      let entries = std::fs::read_dir(dir)?;

      for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some(binary_name) {
          return Ok(path);
        }

        if path.is_dir()
          && let Ok(found) = search_directory(&path, binary_name)
        {
          return Ok(found);
        }
      }

      Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Binary not found",
      ))
    }

    search_directory(base_path, binary_name).map_err(|e| {
      // List directory contents for debugging
      let mut debug_info = format!("Failed to find binary '{}': {}\n", binary_name, e);
      debug_info.push_str("Checked common paths:\n");
      for path in &common_paths {
        debug_info.push_str(&format!("  {:?}\n", path));
      }
      debug_info.push_str("Directory contents:\n");

      fn list_directory_recursive(path: &std::path::Path, prefix: &str, output: &mut String) {
        if let Ok(entries) = std::fs::read_dir(path) {
          for entry in entries.flatten() {
            let entry_path = entry.path();
            let name = entry_path.file_name().unwrap_or_default().to_string_lossy();
            output.push_str(&format!("{}{}\n", prefix, name));

            if entry_path.is_dir() && prefix.len() < 20 {
              // Limit recursion depth
              list_directory_recursive(&entry_path, &format!("{}  ", prefix), output);
            }
          }
        }
      }

      list_directory_recursive(base_path, "", &mut debug_info);
      debug_info
    })
  }
}

impl zed::Extension for EmmyLuaExtension {
  fn new() -> Self {
    Self
  }

  fn language_server_command(
    &mut self,
    language_server_id: &LanguageServerId,
    worktree: &zed::Worktree,
  ) -> Result<zed::Command> {
    let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;

    // Check for custom binary in settings
    if let Some(binary) = settings.binary {
      let command = binary.path.unwrap_or_else(|| "emmylua_ls".to_string());
      let args = binary.arguments.unwrap_or_else(Vec::new);

      return Ok(zed::Command {
        command,
        args,
        env: Default::default(),
      });
    }

    // Install or use the bundled language server
    let server_path = self.check_and_install_server(language_server_id)?;

    // Final verification that the binary exists and is executable
    if !self.binary_exists(&server_path) {
      return Err(format!(
        "Binary not found at expected path: {:?}",
        server_path
      ));
    }

    // Make sure the binary is executable
    zed::make_file_executable(server_path.to_string_lossy().as_ref())?;

    // log_path based on server_path
    let log_dir = server_path.parent().unwrap().parent().unwrap().join("logs");
    std::fs::create_dir_all(log_dir.clone()).map_err(|e| e.to_string())?;

    Ok(zed::Command {
      command: server_path.to_string_lossy().to_string(),
      args: vec![
        "-c".to_string(),
        "stdio".to_string(),
        "--log-level".to_string(),
        "error".to_string(),
      ],
      env: Default::default(),
    })
  }

  fn language_server_workspace_configuration(
    &mut self,
    language_server_id: &LanguageServerId,
    worktree: &Worktree,
  ) -> Result<Option<Value>> {
    let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;

    let Some(settings) = lsp_settings.settings else {
      return Ok(Some(serde_json::json!({})));
    };

    Ok(Some(serde_json::json!({
      "workspace": {
        "library": settings.get("workspace").and_then(|v| v.get("library")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "ignoreDir": settings.get("workspace").and_then(|v| v.get("ignoreDir")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "ignoreGlobs": settings.get("workspace").and_then(|v| v.get("ignoreGlobs")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "workspaceRoots": settings.get("workspace").and_then(|v| v.get("workspaceRoots")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "moduleMap": settings.get("workspace").and_then(|v| v.get("moduleMap")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "encoding": settings.get("workspace").and_then(|v| v.get("encoding")).and_then(|v| v.as_str()).unwrap_or("utf-8"),
        "preloadFileSize": settings.get("workspace").and_then(|v| v.get("preloadFileSize")).and_then(|v| v.as_i64()).unwrap_or(0),
        "enableReindex": settings.get("workspace").and_then(|v| v.get("enableReindex")).and_then(|v| v.as_bool()).unwrap_or(false),
        "reindexDuration": settings.get("workspace").and_then(|v| v.get("reindexDuration")).and_then(|v| v.as_u64()).unwrap_or(5000),
      },
      "completion": {
        "enable": settings.get("completion").and_then(|v| v.get("enable")).and_then(|v| v.as_bool()).unwrap_or(true),
        "callSnippet": settings.get("completion").and_then(|v| v.get("callSnippet")).and_then(|v| v.as_bool()).unwrap_or(false),
        "autoRequire": settings.get("completion").and_then(|v| v.get("autoRequire")).and_then(|v| v.as_bool()).unwrap_or(true),
        "autoRequireFunction": settings.get("completion").and_then(|v| v.get("autoRequireFunction")).and_then(|v| v.as_str()).unwrap_or("require"),
        "autoRequireNamingConvention": settings.get("completion").and_then(|v| v.get("autoRequireNamingConvention")).and_then(|v| v.as_str()).unwrap_or("keep"),
        "autoRequireSeparator": settings.get("completion").and_then(|v| v.get("autoRequireSeparator")).and_then(|v| v.as_str()).unwrap_or("."),
        "baseFunctionIncludesName": settings.get("completion").and_then(|v| v.get("baseFunctionIncludesName")).and_then(|v| v.as_bool()).unwrap_or(true),
        "postfix": settings.get("completion").and_then(|v| v.get("postfix")).and_then(|v| v.as_str()).unwrap_or("@"),
      },
      "diagnostics": {
        "enable": settings.get("diagnostics").and_then(|v| v.get("enable")).and_then(|v| v.as_bool()).unwrap_or(true),
        "globals": settings.get("diagnostics").and_then(|v| v.get("globals")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "globalsRegex": settings.get("diagnostics").and_then(|v| v.get("globalsRegex")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "disable": settings.get("diagnostics").and_then(|v| v.get("disable")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "enables": settings.get("diagnostics").and_then(|v| v.get("enables")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "severity": settings.get("diagnostics").and_then(|v| v.get("severity")).cloned().unwrap_or_else(|| serde_json::json!({})),
        "diagnosticInterval": settings.get("diagnostics").and_then(|v| v.get("diagnosticInterval")).and_then(|v| v.as_u64()).unwrap_or(500),
      },
      "hint": {
        "enable": settings.get("hint").and_then(|v| v.get("enable")).and_then(|v| v.as_bool()).unwrap_or(true),
        "paramHint": settings.get("hint").and_then(|v| v.get("paramHint")).and_then(|v| v.as_bool()).unwrap_or(true),
        "localHint": settings.get("hint").and_then(|v| v.get("localHint")).and_then(|v| v.as_bool()).unwrap_or(true),
        "indexHint": settings.get("hint").and_then(|v| v.get("indexHint")).and_then(|v| v.as_bool()).unwrap_or(true),
        "overrideHint": settings.get("hint").and_then(|v| v.get("overrideHint")).and_then(|v| v.as_bool()).unwrap_or(true),
        "metaCallHint": settings.get("hint").and_then(|v| v.get("metaCallHint")).and_then(|v| v.as_bool()).unwrap_or(true),
        "enumParamHint": settings.get("hint").and_then(|v| v.get("enumParamHint")).and_then(|v| v.as_bool()).unwrap_or(false),
      },
      "runtime": {
        "version": settings.get("runtime").and_then(|v| v.get("version")).and_then(|v| v.as_str()).unwrap_or("LuaLatest"),
        "extensions": settings.get("runtime").and_then(|v| v.get("extensions")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "requireLikeFunction": settings.get("runtime").and_then(|v| v.get("requireLikeFunction")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "requirePattern": settings.get("runtime").and_then(|v| v.get("requirePattern")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "nonstandardSymbol": settings.get("runtime").and_then(|v| v.get("nonstandardSymbol")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "frameworkVersions": settings.get("runtime").and_then(|v| v.get("frameworkVersions")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "special": settings.get("runtime").and_then(|v| v.get("special")).cloned().unwrap_or_else(|| serde_json::json!({})),
        "classDefaultCall": {
          "functionName": settings.get("runtime").and_then(|v| v.get("classDefaultCall")).and_then(|v| v.get("functionName")).and_then(|v| v.as_str()).unwrap_or(""),
          "forceNonColon": settings.get("runtime").and_then(|v| v.get("classDefaultCall")).and_then(|v| v.get("forceNonColon")).and_then(|v| v.as_bool()).unwrap_or(false),
          "forceReturnSelf": settings.get("runtime").and_then(|v| v.get("classDefaultCall")).and_then(|v| v.get("forceReturnSelf")).and_then(|v| v.as_bool()).unwrap_or(false),
        },
      },
      "hover": {
        "enable": settings.get("hover").and_then(|v| v.get("enable")).and_then(|v| v.as_bool()).unwrap_or(true),
        "customDetail": settings.get("hover").and_then(|v| v.get("customDetail")).and_then(|v| v.as_u64()),
      },
      "format": {
        "useDiff": settings.get("format").and_then(|v| v.get("useDiff")).and_then(|v| v.as_bool()).unwrap_or(false),
        "externalTool": settings.get("format").and_then(|v| v.get("externalTool")).cloned(),
        "externalToolRangeFormat": settings.get("format").and_then(|v| v.get("externalToolRangeFormat")).cloned(),
      },
      "doc": {
        "syntax": settings.get("doc").and_then(|v| v.get("syntax")).and_then(|v| v.as_str()).unwrap_or("md"),
        "knownTags": settings.get("doc").and_then(|v| v.get("knownTags")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "privateName": settings.get("doc").and_then(|v| v.get("privateName")).cloned().unwrap_or_else(|| serde_json::json!([])),
        "rstDefaultRole": settings.get("doc").and_then(|v| v.get("rstDefaultRole")).and_then(|v| v.as_str()),
        "rstPrimaryDomain": settings.get("doc").and_then(|v| v.get("rstPrimaryDomain")).and_then(|v| v.as_str()),
      },
      "codeLens": {
        "enable": settings.get("codeLens").and_then(|v| v.get("enable")).and_then(|v| v.as_bool()).unwrap_or(true),
      },
      "semanticTokens": {
        "enable": settings.get("semanticTokens").and_then(|v| v.get("enable")).and_then(|v| v.as_bool()).unwrap_or(true),
        "renderDocumentationMarkup": settings.get("semanticTokens").and_then(|v| v.get("renderDocumentationMarkup")).and_then(|v| v.as_bool()).unwrap_or(false),
      },
      "signature": {
        "detailSignatureHelper": settings.get("signature").and_then(|v| v.get("detailSignatureHelper")).and_then(|v| v.as_bool()).unwrap_or(true),
      },
      "references": {
        "enable": settings.get("references").and_then(|v| v.get("enable")).and_then(|v| v.as_bool()).unwrap_or(true),
        "fuzzySearch": settings.get("references").and_then(|v| v.get("fuzzySearch")).and_then(|v| v.as_bool()).unwrap_or(true),
        "shortStringSearch": settings.get("references").and_then(|v| v.get("shortStringSearch")).and_then(|v| v.as_bool()).unwrap_or(false),
      },
      "documentColor": {
        "enable": settings.get("documentColor").and_then(|v| v.get("enable")).and_then(|v| v.as_bool()).unwrap_or(true),
      },
      "inlineValues": {
        "enable": settings.get("inlineValues").and_then(|v| v.get("enable")).and_then(|v| v.as_bool()).unwrap_or(true),
      },
      "codeAction": {
        "insertSpace": settings.get("codeAction").and_then(|v| v.get("insertSpace")).and_then(|v| v.as_bool()).unwrap_or(false),
      },
      "strict": {
        "arrayIndex": settings.get("strict").and_then(|v| v.get("arrayIndex")).and_then(|v| v.as_bool()).unwrap_or(true),
        "docBaseConstMatchBaseType": settings.get("strict").and_then(|v| v.get("docBaseConstMatchBaseType")).and_then(|v| v.as_bool()).unwrap_or(false),
        "metaOverrideFileDefine": settings.get("strict").and_then(|v| v.get("metaOverrideFileDefine")).and_then(|v| v.as_bool()).unwrap_or(true),
        "requirePath": settings.get("strict").and_then(|v| v.get("requirePath")).and_then(|v| v.as_bool()).unwrap_or(false),
        "typeCall": settings.get("strict").and_then(|v| v.get("typeCall")).and_then(|v| v.as_bool()).unwrap_or(false),
      },
      "resource": {
        "paths": settings.get("resource").and_then(|v| v.get("paths")).cloned().unwrap_or_else(|| serde_json::json!([])),
      },
    })))
  }
}

zed::register_extension!(EmmyLuaExtension);
