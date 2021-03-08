use gtk4::prelude::*;
use gtk4::{
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib::{timeout_add, timeout_add_seconds},
    prelude::*,
    Image,
};
use rand::prelude::*;
use x11::xlib::{
    XConfigureWindow, XDefaultRootWindow, XFetchName, XFlush, XGetGeometry, XGetImage,
    XMoveResizeWindow, XQueryTree,
};

use std::{
    default,
    env::args,
    ffi::{c_void, CStr, CString},
    os::raw::c_char,
    process::Command,
    ptr::{self, null},
    thread,
    time::{Duration, Instant},
};

const FPS: u128 = 5;
const SCREEN_ID: &str = ":55";
const PROGRAM: &str = "gedit";

unsafe fn on_activate(application: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::new(application);
    window.set_title(Some("espy"));
    window.set_default_size(600, 400);

    let drawing_area = gtk4::DrawingArea::new();
    // let buf = Pixbuf::new(Colorspace::Rgb, true, 8, 1920, 1080).unwrap();
    // buf.fill(1703705);
    // image.set_from_pixbuf(Some(&buf));

    window.set_child(Some(&drawing_area));

    window.show();

    // create a new display with Xvfb
    thread::spawn(move || {
        let mut child = Command::new("Xvfb")
            .args(&[SCREEN_ID, "-shmem", "-screen", "0", "1920x1080x24"])
            .spawn()
            .unwrap();
        println!("Started screen {} with xvfb", SCREEN_ID);
        child.wait().unwrap();
    });

    thread::sleep(Duration::from_secs(1));

    // start program on display
    thread::spawn(move || {
        let mut child = Command::new(PROGRAM)
            .env("DISPLAY", SCREEN_ID)
            .env("GTK_THEME", "Adwaita:dark")
            .spawn()
            .unwrap();
        println!("Started {}", PROGRAM);
        child.wait().unwrap();
    });

    thread::sleep(Duration::from_secs(1));

    // get display with xlib
    let screen_id = CString::new(SCREEN_ID).unwrap();
    let display = x11::xlib::XOpenDisplay(screen_id.as_ptr() as *const c_char);
    if display == ptr::null_mut() {
        panic!("display is null!");
    }

    // diagnosis
    let connection_number = x11::xlib::XConnectionNumber(display);
    println!(
        "Connected to {} (connection number: {})",
        SCREEN_ID, connection_number
    );

    // get screen 0
    let screen = x11::xlib::XDefaultScreenOfDisplay(display);
    println!("Default screen = {:?}", screen);

    // root window
    let root = x11::xlib::XRootWindowOfScreen(screen);
    println!("Root window = {}", root);

    let mut children = ptr::null_mut();
    let mut rr = Box::new(0);
    let mut pr = Box::new(0);
    let mut child_count: u32 = 0;
    x11::xlib::XQueryTree(
        display,
        root,
        rr.as_mut(),
        pr.as_mut(),
        &mut children,
        &mut child_count,
    );

    let children = Vec::from_raw_parts(children, child_count as usize, child_count as usize);

    {
        let drawing_area = drawing_area.clone();
        gtk4::glib::timeout_add_local(Duration::from_millis(1000 / (FPS as u64)), move || {
            drawing_area.queue_draw();

            Continue(true)
        });
    }

    drawing_area.set_draw_func(move |area, context, _, _| {
        println!("draw begin");

        let window_image = x11::xlib::XGetImage(
            display,
            root,
            0,
            0,
            1920,
            1080,
            u64::MAX,
            x11::xlib::ZPixmap,
        );

        // let data: &mut [u8] = std::slice::from_raw_parts_mut(
        //     window_image.data as *mut u8,
        //     (window_image.height * window_image.bytes_per_line) as usize,
        // );
        // IS THIS IN BGR?

        // context.set_source_rgb(1.0, 1.0, 1.0);
        // context.rectangle(0.0, 0.0, 1920.0, 1080.0);
        // context.fill();

        // let pixbuf = Pixbuf::from_mut_slice(
        //     data,
        //     Colorspace::Rgb,
        //     true,
        //     8,
        //     1920,
        //     1080,
        //     window_image.bytes_per_line,
        // );
        // context.set_source_pixbuf(&pixbuf, 0.0, 0.0);
        // context.rectangle(0.0, 0.0, 1920.0, 1080.0);
        // context.fill();
        for x in 0..1920 {
            for y in 0..1080 {
                let pixel = x11::xlib::XGetPixel(window_image, x, y);

                let b = pixel as u8;
                let g = (pixel >> 8) as u8;
                let r = (pixel >> 16) as u8;
                let a = (pixel >> 24) as u8;

                context.set_source_rgb(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);

                context.rectangle(x as f64, y as f64, 1.0, 1.0);
                context.fill();
            }
        }
    });
    // (1000 / FPS, || {

    // })

    // main_loop(image.clone());
}

// for &child in children.iter() {
//     let mut window_name_ptr: *mut i8 = ptr::null_mut();
//     x11::xlib::XFetchName(display, child, &mut window_name_ptr);

//     if window_name_ptr == ptr::null_mut() {
//         continue;
//     };

//     let window_name = CStr::from_ptr(window_name_ptr);
//     let window_image =
//         x11::xlib::XGetImage(display, root, 0, 0, 200, 200, u64::MAX, x11::xlib::ZPixmap);

//     if window_image == ptr::null_mut() {
//         continue;
//     }
//     println!(
//         "{}: window name = {}, window image = {:?}",
//         child,
//         window_name.to_str().unwrap(),
//         window_image
//     );
//     x11::xlib::XFree(window_name_ptr as *mut c_void);

//     for x in 0..200 {
//         for y in 0..200 {
//             let pixel = x11::xlib::XGetPixel(window_image, x, y);

//             let r = pixel as u8;
//             let g = (pixel >> 8) as u8;
//             let b = (pixel >> 16) as u8;
//             let a = (pixel >> 24) as u8;

//             if pixel != 0 {
//                 println!(
//                     "pixel = {}, r = {}, g = {}, b = {}, a = {}",
//                     pixel, r, g, b, a
//                 );
//             }

//             context.set_source_rgb(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);

//             context.rectangle(x as f64, y as f64, 1.0, 1.0);
//             context.fill();
//         }
//     }

//     break;
// }

fn main_loop(image: Image) {
    unsafe {
        // get display with xlib
        let screen_id = CString::new(SCREEN_ID).unwrap();
        let display = x11::xlib::XOpenDisplay(screen_id.as_ptr() as *const c_char);
        if display == ptr::null_mut() {
            panic!("display is null!");
        }

        // diagnosis
        let connection_number = x11::xlib::XConnectionNumber(display);
        println!(
            "Connected to {} (connection number: {})",
            SCREEN_ID, connection_number
        );

        // get screen 0
        let screen = x11::xlib::XDefaultScreenOfDisplay(display);
        println!("Default screen = {:?}", screen);

        // root window
        let root = x11::xlib::XRootWindowOfScreen(screen);
        println!("Root window = {}", root);

        let mut children = ptr::null_mut();
        let mut rr = Box::new(0);
        let mut pr = Box::new(0);
        let mut child_count: u32 = 0;
        x11::xlib::XQueryTree(
            display,
            root,
            rr.as_mut(),
            pr.as_mut(),
            &mut children,
            &mut child_count,
        );
        let children = Vec::from_raw_parts(children, child_count as usize, child_count as usize);
        // children.set_len(child_count as usize);
        println!("Found {} children, {:?}", child_count, children);

        for child in children {
            let mut window_name_ptr: *mut i8 = ptr::null_mut();
            x11::xlib::XFetchName(display, child, &mut window_name_ptr);

            if window_name_ptr == ptr::null_mut() {
                continue;
            };

            let window_name = CStr::from_ptr(window_name_ptr);
            println!("{}: window name = {}", child, window_name.to_str().unwrap());
            x11::xlib::XFree(window_name_ptr as *mut c_void);

            let window_image =
                x11::xlib::XGetImage(display, child, 0, 0, 200, 200, u64::MAX, x11::xlib::ZPixmap);

            let buf = Pixbuf::new(Colorspace::Rgb, true, 8, 200, 200).unwrap();
            for x in 0..200 {
                for y in 0..200 {
                    let pixel = x11::xlib::XGetPixel(window_image, x, y);

                    buf.put_pixel(
                        x as u32,
                        y as u32,
                        pixel as u8,
                        (pixel << 8) as u8,
                        (pixel << 16) as u8,
                        (pixel << 24) as u8,
                    )
                }
            }
            image.set_from_pixbuf(Some(&buf));
        }

        // for i in 0..child_count {
        //     let child = *children.offset(i as isize);
        //     println!("child = {}", child);

        //     // get window name
        //     let mut window_name_ptr: Box<*mut i8> = Box::new(ptr::null_mut());
        //     x11::xlib::XFetchName(display, child, window_name_ptr.as_mut());
        //     let wn = CString::from_raw(*window_name_ptr);
        //     let window_name = String::from(wn.as_c_str().to_owned().to_str().unwrap());
        //     // x11::xlib::XFree(*window_name_ptr as *mut c_void);

        //     println!("{}: child = {}, window_name = {}", i, child, window_name);
        // }
    }

    let mut accumulator: u128 = 0;
    let mut time = Instant::now();

    loop {
        let current_time = Instant::now();
        accumulator += (current_time - time).as_nanos();
        time = current_time;

        if accumulator > 1_000_000_000 / FPS {
            accumulator -= 1_000_000_000 / FPS;
            println!("frame");
            thread::sleep(Duration::from_millis(1));
        }
    }
}

fn main() {
    let application =
        gtk4::Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
            .expect("Initialization failed...");

    application.connect_activate(|app| unsafe {
        on_activate(app);
    });
    application.run(&args().collect::<Vec<_>>());
}

fn x11hacking() {
    let id = ":73";

    let mut xvfb = Command::new("Xvfb")
        .args(&[id, "-shmem", "-screen", "0", "1920x1080x24"])
        .spawn()
        .unwrap();

    thread::sleep(Duration::from_millis(1000));

    let mut boop = Command::new("gedit").env("DISPLAY", id).spawn().unwrap();

    thread::sleep(Duration::from_millis(1000));

    unsafe {
        let display_code = CString::new(id).unwrap();
        let display = x11::xlib::XOpenDisplay(display_code.as_ptr() as *const c_char);
        if display == ptr::null_mut() {
            panic!("display is null!");
        }

        let conn_number = x11::xlib::XConnectionNumber(display);
        println!("conn_number = {}", conn_number);

        // let screen = x11::xlib::XScreenOfDisplay(display, 0);
        let screen = x11::xlib::XDefaultScreenOfDisplay(display);
        let root = x11::xlib::XRootWindowOfScreen(screen);

        // let mut children: Vec<Box<u64>> = Vec::with_capacity(1000);
        let mut children: Box<*mut u64> = Box::new(ptr::null_mut());
        let mut rr = Box::new(0);
        let mut pr = Box::new(0);
        let mut count = 0;
        let tree = x11::xlib::XQueryTree(
            display,
            root,
            rr.as_mut(),
            pr.as_mut(),
            children.as_mut(),
            &mut count,
        );
        // children.set_len(count as usize);

        println!("root = {}", root);
        println!("count = {}", count);
        println!("children = {:?}", children);

        let mut rootr = Box::new(0);
        let mut x = Box::new(0);
        let mut y = Box::new(0);
        let mut width = Box::new(0);
        let mut height = Box::new(0);
        let mut border_width = Box::new(0);
        let mut depth = Box::new(0);

        x11::xlib::XGetGeometry(
            display,
            root,
            rootr.as_mut(),
            x.as_mut(),
            y.as_mut(),
            width.as_mut(),
            height.as_mut(),
            border_width.as_mut(),
            depth.as_mut(),
        );
        println!(
            "rootr = {}, x = {}, y = {}, width = {}, height = {}, border width = {}, depth = {}",
            rootr, x, y, width, height, border_width, depth
        );

        // let mut changes = Box::new(x11::xlib::XWindowChanges {
        //     x: 0,
        //     y: 0,
        //     width: 400,
        //     height: 400,
        //     border_width: 0,
        //     sibling: root,
        //     stack_mode: 0,
        // });

        // let flags = x11::xlib::CWX
        //     | x11::xlib::CWY
        //     | x11::xlib::CWWidth
        //     | x11::xlib::CWHeight
        //     | x11::xlib::CWBorderWidth;

        // let result = x11::xlib::XMoveResizeWindow(display, root, 0, 0, 1920, 1080);

        for i in 0..count {
            let child = *children.offset(i as isize);
            println!("child = {}", child);
            x11::xlib::XGetGeometry(
                display,
                root,
                rootr.as_mut(),
                x.as_mut(),
                y.as_mut(),
                width.as_mut(),
                height.as_mut(),
                border_width.as_mut(),
                depth.as_mut(),
            );
            println!(
                "rootr = {}, x = {}, y = {}, width = {}, height = {}, border width = {}, depth = {}",
                rootr, x, y, width, height, border_width, depth
            );

            {
                let mut children = Vec::with_capacity(1000);
                let mut rr = Box::new(0);
                let mut pr = Box::new(0);
                let mut count = 0;
                let tree = x11::xlib::XQueryTree(
                    display,
                    child,
                    rr.as_mut(),
                    pr.as_mut(),
                    children.as_mut_ptr(),
                    &mut count,
                );
                children.set_len(count as usize);

                println!("root = {}", root);
                println!("count = {}", count);
                println!("children = {:?}", children);
            }

            // let result = x11::xlib::XMoveResizeWindow(display, child, 0, 0, 1920, 1080);
            // println!("child = {}, result = {}", child, result);
            let mut params = Box::new(x11::xlib::XWindowChanges {
                width: 1920,
                height: 1080,
                x: 0,
                y: 0,
                border_width: 0,
                sibling: 0,
                stack_mode: 0,
            });
            let result = x11::xlib::XConfigureWindow(
                display,
                child,
                (x11::xlib::CWWidth | x11::xlib::CWHeight) as u32,
                params.as_mut(),
            );

            x11::xlib::XGetGeometry(
                display,
                root,
                rootr.as_mut(),
                x.as_mut(),
                y.as_mut(),
                width.as_mut(),
                height.as_mut(),
                border_width.as_mut(),
                depth.as_mut(),
            );
            println!(
                "rootr = {}, x = {}, y = {}, width = {}, height = {}, border width = {}, depth = {}",
                rootr, x, y, width, height, border_width, depth
            );
        }

        let result = x11::xlib::XSync(display, 1);
        x11::xlib::XFlush(display);
        println!("result = {}", result);
    }

    xvfb.wait().unwrap();
}
