#![allow(unused_imports)]
use downloaders::commands::{
    mangadex_get_chapter_images, mangadex_get_chapters, mangadex_get_favorite_languages,
    mangadex_get_manga_by_id, mangadex_get_manga_by_url, mangadex_search,
    mangafire_get_chapter_images, mangafire_get_chapters, mangafire_get_favorite_languages,
    mangafire_get_manga_by_id, mangafire_get_manga_by_url, mangafire_search,
};
use downloaders::DownloaderState;
use extensions::commands::{
    _extension::{
        extension_get_chapter_images, extension_get_chapters, extension_get_languages,
        extension_search,
    },
    _extension_control::{load_extension, reload_extension, unload_extension},
    _information::{
        get_extension_info, get_extensions_directory, install_extension_from_file,
        list_installed_extensions, list_loaded_extensions, uninstall_extension,
    },
    _state::ExtensionState,
};
use tauri::Manager;
use utils::hashmap::{get_data, set_data};
use utils::request::{get_aniplay_chapters, get_aniplay_episode, get_base64_image};
// use webview2_com::{
//     Microsoft::Web::WebView2::Win32::{
//         ICoreWebView2WebResourceRequest, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
//     },
//     WebResourceRequestedEventHandler,
// };
// use windows::core::HSTRING;
// use windows::Win32::System::Com::MachineGlobalObjectTableRegistrationToken;

mod downloaders;
mod extensions;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .setup(|app| {
            // Inicializar sistema de extensões
            // Usar diretório de dados da aplicação (AppData no Windows)
            let extensions_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir")
                .join("extensions");

            // Criar diretório se não existir
            if !extensions_dir.exists() {
                std::fs::create_dir_all(&extensions_dir)
                    .expect("Failed to create extensions directory");
            }

            let extension_state =
                ExtensionState::new(extensions_dir).expect("Failed to initialize extensions");

            app.manage(extension_state);
            app.manage(DownloaderState::default());

            // Desktop-specific setup
            #[cfg(desktop)]
            {
                let window = app.get_webview_window("main").unwrap();
                let args: Vec<String> = std::env::args().collect();
                if args.contains(&"--flag1".to_string()) {
                    window.hide().unwrap();
                }
                // #[cfg(target_os = "windows")]
                // {
                //     window
                //         .with_webview(|webview| unsafe {
                //             let core = webview.controller().CoreWebView2().unwrap();
                //             core.AddWebResourceRequestedFilter(
                //                 &HSTRING::from("*"),
                //                 COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                //             )
                //             .unwrap();
                //             let mut _token: MachineGlobalObjectTableRegistrationToken =
                //                 MachineGlobalObjectTableRegistrationToken::default();
                //             core.add_WebResourceRequested(
                //                 &WebResourceRequestedEventHandler::create(Box::new(
                //                     move |_webview, args| {
                //                         if let Some(args) = args {
                //                             let request: ICoreWebView2WebResourceRequest =
                //                                 args.Request().unwrap();
                //                             request
                //                                 .Headers()
                //                                 .unwrap()
                //                                 .SetHeader(
                //                                     &HSTRING::from("Origin"),
                //                                     &HSTRING::from("https://github.com/"),
                //                                 )
                //                                 .unwrap();
                //                         }
                //                         Ok(())
                //                     },
                //                 )),
                //                 &mut _token as *mut _ as *mut i64,
                //             )
                //             .unwrap();
                //         })
                //         .unwrap();
                // }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Comandos de utilidades
            set_data,
            get_data,
            get_aniplay_chapters,
            get_aniplay_episode,
            get_base64_image,
            // Comandos MangaDex (legado - será migrado para extensão)
            mangadex_search,
            mangadex_get_favorite_languages,
            mangadex_get_chapters,
            mangadex_get_chapter_images,
            mangadex_get_manga_by_url,
            mangadex_get_manga_by_id,
            // Comandos MangaFire (legado - será migrado para extensão)
            mangafire_search,
            mangafire_get_favorite_languages,
            mangafire_get_chapters,
            mangafire_get_chapter_images,
            mangafire_get_manga_by_url,
            mangafire_get_manga_by_id,
            // Comandos de extensões WASM (novo sistema modular)
            list_installed_extensions,
            list_loaded_extensions,
            load_extension,
            unload_extension,
            reload_extension,
            get_extension_info,
            extension_search,
            extension_get_chapters,
            extension_get_chapter_images,
            extension_get_languages,
            install_extension_from_file,
            uninstall_extension,
            get_extensions_directory,
        ])
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_cache::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build());

    #[cfg(mobile)]
    {
        builder = builder.setup(|app| {
            app.handle().plugin(tauri_plugin_app_events::init())?;
            Ok(())
        });
    }

    #[cfg(desktop)]
    {
        builder = builder
            // .plugin(tauri_plugin_drpc::init())
            .plugin(tauri_plugin_cli::init())
            .plugin(tauri_plugin_clipboard::init())
            .plugin(tauri_plugin_positioner::init())
            .plugin(tauri_plugin_prevent_default::debug())
            .plugin(tauri_plugin_window_state::Builder::default().build())
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                let win = app.get_webview_window("main").unwrap();
                win.show().unwrap();
                win.set_focus().unwrap();
            }))
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec!["--flag1", "--flag2"]),
            ))
            .plugin(tauri_plugin_global_shortcut::Builder::new().build());
    }

    builder.run(tauri::generate_context!()).unwrap()
}
