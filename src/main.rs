// jkcoxson

use rusty_libimobiledevice::{
    error::InstProxyError,
    instproxy::InstProxyClient,
    libimobiledevice,
    plist::{Plist, PlistDictIter},
};

fn main() {
    println!("Starting JIT diagnostic program");
    let mut devices = match libimobiledevice::get_devices() {
        Ok(devices) => devices,
        Err(e) => {
            println!("Error getting devices from the muxer: {:?}", e);
            pause();
            return;
        }
    };
    println!("Found {} devices", devices.len());
    if devices.len() == 0 {
        println!("No devices found");
        pause();
        return;
    }
    println!("Selecting first device...");
    let device = devices.remove(0);
    let instproxy_client = match device.new_instproxy_client("idevicedebug".to_string()) {
        Ok(instproxy) => {
            println!("Successfully started instproxy");
            instproxy
        }
        Err(e) => {
            println!("Error starting instproxy: {:?}", e);
            if e == InstProxyError::UnknownError {
                println!("This can be due to bad pairing files. Delete {}.plist from your OS's respective folder and re-pair your device.", device.udid);
            }
            pause();
            return;
        }
    };
    let mut client_opts = InstProxyClient::options_new();
    InstProxyClient::options_add(
        &mut client_opts,
        vec![("ApplicationType".to_string(), Plist::new_string("Any"))],
    );
    InstProxyClient::options_set_return_attributes(
        &mut client_opts,
        vec![
            "CFBundleIdentifier".to_string(),
            "CFBundleExecutable".to_string(),
        ],
    );
    let lookup_results = match instproxy_client.lookup(vec![], client_opts) {
        Ok(apps) => {
            println!("Successfully looked up apps");
            apps
        }
        Err(e) => {
            println!("Error looking up apps: {:?}", e);
            pause();
            return;
        }
    };

    println!("Apps installed:");

    let mut lookup_results_iter: PlistDictIter = lookup_results.into();

    loop {
        if let Some((key, _)) = lookup_results_iter.next_item() {
            if key.starts_with("com.apple") {
                continue;
            }
            println!("{:?}", key);
        } else {
            break;
        }
    }

    let _ = match device.new_debug_server("jitdiag") {
        Ok(d) => {
            println!("Successfully started debug server");
            d
        }
        Err(e) => {
            println!("Error starting debug server: {:?}", e);
            println!("This can be caused by not having the DMG mounted. You can do that either through AltServer or running `ideviceimagemounter`.");
            pause();
            return;
        }
    };

    println!("All tests completede succesfully!");
    pause();
}

fn pause() {
    let mut stdout = std::io::stdout();
    std::io::Write::write(&mut stdout, b"Press Enter to continue...").unwrap();
    std::io::Write::flush(&mut stdout).unwrap();
    std::io::Read::read(&mut std::io::stdin(), &mut [0]).unwrap();
}
