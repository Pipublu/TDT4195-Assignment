// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::{result, thread};
use std::sync::{Mutex, Arc, RwLock};
use gl::FALSE;
use rand::Rng;


mod shader;
mod util;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

struct Camera {
    position: glm::Vec3,
    rotation: glm::Vec2,
}

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  byte_size_of_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

// Creates a vector of vertices for generating a circle in the x,y plane
// Given a start position, radius, and amount of vertices
fn create_circle_vertices(radius: f32, num_vertices: usize, center: Option<(f32, f32, f32)>) -> Vec<f32> {
    let mut vertices: Vec<f32> = Vec::with_capacity(num_vertices * 3);

    // If center is None, use (0,0,0)
    let (start_x, start_y, start_z) = center.unwrap_or((0.0, 0.0, 0.0));

    // Generate vertices
    for i in 0..num_vertices {
        let angle: f32 = 2.0 * std::f32::consts::PI * i as f32 / num_vertices as f32;

        let x: f32 = start_x + radius * angle.cos();
        let y: f32 = start_y + radius * angle.sin();
        let z: f32 = start_z;

        vertices.push(x);
        vertices.push(y);
        vertices.push(z);
    }

    vertices
} 

// Generates a vector containing rgba values. Creates as many as count, alpha is set to 1.
fn generate_colors(count: usize) -> Vec<f32> {
    let mut color_vec: Vec<f32> = Vec::with_capacity(count * 4);

    let mut rng = rand::thread_rng();

    for i in 0..count {
        let r: f32 = rng.gen_range(0.0..1.0);
        let g: f32 = rng.gen_range(0.0..1.0);
        let b: f32 = rng.gen_range(0.0..1.0);

        color_vec.push(r);
        color_vec.push(g);
        color_vec.push(b);
        color_vec.push(1.0);
    }

   color_vec
}


// == // Generate your VAO here
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>) -> u32 {
    // Generate VAO and bind it
    let mut vertex_array:  u32 = 0;
    gl::GenVertexArrays(1, &mut vertex_array);
    gl::BindVertexArray(vertex_array);

    // Generate VBO and bind it
    let mut vertex_buffer: u32 = 0;
    gl::GenBuffers(1, &mut vertex_buffer);
    gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);

    // Fill it with data
    let size = vertices.len() * std::mem::size_of::<f32>();
    gl::BufferData(
            gl::ARRAY_BUFFER,
            size as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW
        );
        // Configure and enable VAP
    gl::VertexAttribPointer(
            1,
             3,
             gl::FLOAT,
             gl::FALSE,
             3 * std::mem::size_of::<f32>() as i32,
             std::ptr::null()
        );
    gl::EnableVertexArrayAttrib(vertex_array, 1);

    // Generate IBO and bind it
    let mut index_buffer: u32 = 0;
    let indices_size = indices.len() * std::mem::size_of::<u32>();
    
    gl::GenBuffers(1, &mut index_buffer);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);

    // Fill it
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        indices_size as isize,
        indices.as_ptr() as *const _,
        gl::STATIC_DRAW
    );

    vertex_array
}

// Create vao with color specified
unsafe fn create_vao_w_colors(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>) -> Result<u32, String> {
    if vertices.len() / 3 != colors.len() / 4 {
        return Err(format!("Must contain one color per vertice! Vertices: {}, Colors: {}", vertices.len(), colors.len()));
    }
    // Generate VAO and bind it
    let mut vertex_array:  u32 = 0;
    gl::GenVertexArrays(1, &mut vertex_array);
    gl::BindVertexArray(vertex_array);

    // Generate VBO and bind it
    let mut vertex_buffer: u32 = 0;
    gl::GenBuffers(1, &mut vertex_buffer);
    gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);

    // Fill it with data
    let size = vertices.len() * std::mem::size_of::<f32>();
    gl::BufferData(
            gl::ARRAY_BUFFER,
            size as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW
        );

    // Configure and enable VAP
    gl::VertexAttribPointer(
            1,
             3,
             gl::FLOAT,
             gl::FALSE,
             3 * std::mem::size_of::<f32>() as i32,
             std::ptr::null()
        );
    gl::EnableVertexArrayAttrib(vertex_array, 1);

    
    // Generate VBO for color and bind it
    let size = colors.len() * std::mem::size_of::<f32>();
    let mut color_buffer: u32 = 0;
    gl::GenBuffers(1, &mut color_buffer);
    gl::BindBuffer(gl::ARRAY_BUFFER, color_buffer);

    // Fill VBO with colors
    gl::BufferData(
            gl::ARRAY_BUFFER,
            size as isize,
            colors.as_ptr() as *const _,
            
            gl::STATIC_DRAW
        );

    gl::VertexAttribPointer(
        2, 
        4,
        gl::FLOAT,
        gl::FALSE,
        4 * std::mem::size_of::<f32>() as i32,
        std::ptr::null()
        );
    gl::EnableVertexAttribArray( 2);

    // Generate IBO and bind it
    let mut index_buffer: u32 = 0;
    let indices_size = indices.len() * std::mem::size_of::<u32>();
    
    gl::GenBuffers(1, &mut index_buffer);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);

    // Fill it
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        indices_size as isize,
        indices.as_ptr() as *const _,
        gl::STATIC_DRAW
    );

    Ok(vertex_array)
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // Set up VAO
       
      let vertices: Vec<f32> = vec![
        -0.9, -0.2, 0.0,
        -0.5, -0.3, 0.0,
        -0.7,  0.1, 0.0,

        -0.2, -0.3, 0.0,
        0.2, -0.3, 0.0,
        0.0,  0.1, 0.0,

        0.5, -0.2, 0.0,
        0.9, -0.2, 0.0,
        0.7,  0.1, 0.0,
    ];

        let indices: Vec<u32> = vec![
            0, 1, 2,
            3, 4, 5,
            6, 7, 8
        ];
        // let colors = generate_colors(vertices.len() / 3);

        let colors: Vec<f32> = vec![
            // green
            0.68, 0.984, 0.0, 0.5,
            0.68, 0.984, 0.0, 0.5,
            0.68, 0.984, 0.0, 0.5,

            // pink
            1.0, 0.0, 0.396, 0.5,
            1.0, 0.0, 0.396, 0.5,
            1.0, 0.0, 0.396, 0.5,

            // blue
            0.173, 0.161, 1.0, 0.5,
            0.173, 0.161, 1.0, 0.5,
            0.173, 0.161, 1.0, 0.5,
        ];

        
        let my_vao = unsafe {
            match create_vao_w_colors(&vertices, &indices, &colors) {
                Ok(result) => result,
                Err(error) => {
                    println!("Error: {}", error);
                    std::process::exit(1);
                }
            }
        };

        //let my_vao = unsafe {create_vao(&vertices, &indices)};

        /*
        const NUM_VERTICES: usize = 36;
        let circle_vertices: Vec<f32> = create_circle_vertices(0.2, NUM_VERTICES, None);

        let circle_indices: Vec<u32> = (0..NUM_VERTICES as u32).collect();

        let circle_vao: u32 = unsafe {create_vao(&circle_vertices, &circle_indices)};
         */


        // == // Set up your shaders here

        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.vert")
                .attach_file("./shaders/simple.frag")
                .link()
        };

        unsafe {
            simple_shader.activate();
        }

        // Get time location from shader
        let time_location = unsafe {
            gl::GetUniformLocation(simple_shader.program_id, b"time\0".as_ptr() as *const i8)
        };

        // Get matrix location from shader
        let matrix_location = unsafe {
            gl::GetUniformLocation(simple_shader.program_id, b"matrix\0".as_ptr() as *const i8)
        };

        // Prepare matrices
        let mut translation_matrix:glm::Mat4;
        let mut rotation_matrix:glm::Mat4 ;
        let mut pitch_matrix:glm::Mat4 ;
        let mut yaw_matrix:glm::Mat4 ;
        let mut transformation_matrix:glm::Mat4;

        let projection_matrix: glm::Mat4 = glm::perspective(window_aspect_ratio, 90.0, 1.0, 100.0);
        let mut matrix:glm::Mat4;

        let mut camera: Camera = Camera {
            position: glm::vec3(0.0, 0.0, 2.0),
            rotation: glm::vec2(0.0, 0.0),
        };
 
        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe { gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32); }
                }
            }
            translation_matrix = glm::Mat4::identity();
            pitch_matrix = glm::Mat4::identity();
            yaw_matrix = glm::Mat4::identity();

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html
                        VirtualKeyCode::Up => {
                            camera.rotation[0] -= delta_time;
                        }
                        VirtualKeyCode::Down => {
                            camera.rotation[0] += delta_time;
                        }
                        VirtualKeyCode::Right => {
                            camera.rotation[1] += delta_time;
                        }
                        VirtualKeyCode::Left => {
                            camera.rotation[1] -= delta_time;
                        }
                        VirtualKeyCode::W => {
                            camera.position[1] += delta_time;
                        }
                        VirtualKeyCode::A => {
                            camera.position[0]  -= delta_time;
                        }
                        VirtualKeyCode::S => {
                            camera.position[1]  -= delta_time;
                        }
                        VirtualKeyCode::D => {
                            camera.position[0]  += delta_time;
                        }
                        VirtualKeyCode::Space => {
                            camera.position[2]  -= delta_time;
                        }
                        VirtualKeyCode::LShift => {
                            camera.position[2]  += delta_time;
                        }

                        // default handler:
                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)
            translation_matrix[(0, 3)] = -camera.position[0];
            translation_matrix[(1, 3)] = -camera.position[1];
            translation_matrix[(2, 3)] = -camera.position[2];

             // Pitch rotation
            yaw_matrix[(1, 1)] = camera.rotation[0].cos();
            yaw_matrix[(1, 2)] = -camera.rotation[0].sin();
            yaw_matrix[(2, 1)] = camera.rotation[0].sin();
            yaw_matrix[(2, 2)] = camera.rotation[0].cos();

            // Yaw rotation
            pitch_matrix[(0, 0)] = camera.rotation[1].cos();
            pitch_matrix[(0, 2)] = camera.rotation[1].sin();
            pitch_matrix[(2, 0)] = -camera.rotation[1].sin();
            pitch_matrix[(2, 2)] = camera.rotation[1].cos();

            rotation_matrix = yaw_matrix * pitch_matrix;
            transformation_matrix = rotation_matrix * translation_matrix;
            matrix = projection_matrix * transformation_matrix;

            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);


                // == // Issue the necessary gl:: commands to draw your scene here
                // Send uniform varibles to vertex shader
                gl::UseProgram(simple_shader.program_id);
                gl::Uniform1f(time_location, elapsed);
                gl::UniformMatrix4fv(matrix_location,1, FALSE, matrix.as_ptr());
                
                // Draw triangles
                gl::BindVertexArray(my_vao);
                gl::DrawElements(
                    gl::TRIANGLES,
                    indices.len() as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );

            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });


    // == //
    // == // From here on down there are only internals.
    // == //


    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
                println!("New window size received: {}x{}", physical_size.width, physical_size.height);
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => { *control_flow = ControlFlow::Exit; }
                    Q      => { *control_flow = ControlFlow::Exit; }
                    _      => { }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => { }
        }
    });
}
