<div align="center">

# Multithread Raycaster Game

</div>


## ✨Features 
A Prototype game made in Rust featuring:
- Multithreading by Using the [Rayon](https://crates.io/crates/rayon) crate which allows multithreaded iterators.
- Multithreaded Wall Raycasting by using DDA Algorithm Derived from [Lodev Raycasting Tutorial](https://lodev.org/cgtutor/raycasting.html), special thanks to him as it helped a lot making this project.
- Multithreaded Ceiling and Floor Raycasting, also Derived from [Lodev Raycasting Tutorial](https://lodev.org/cgtutor/raycasting.html).
- Multithreaded Sprite Raycasting, same as above.
- Textures rendering by using [Image](https://crates.io/crates/image) crate, (Still WIP, Currently only 64x64 Textures work).
- Movement and First Person Camera including pitch and crouch and jump for movement, Uses the [Winit](https://crates.io/crates/winit) crate for Event Handling and Window Creation.
- Utilizes [Softbuffer](https://crates.io/crates/softbuffer) crate for Software Pixel Buffer.

The idea for the project is to create a full featuring Raycaster game with most of the features of famous raycasting games like Wolfstein 3D and ShadowCaster but with Newest CPU features like multithreading and new processors architecture to make it possible to run on high resolutions with acceptable performance.

## 💽Building

NOTE: Rust must be installed, you can use rustup to install toolchain.

Download the ZIP of the source. 
Extract to a temporary folder.
Open Console or Command Prompt on the folder:
```
$ cargo build -r
```
Now you move to the directory where the release is:
```
$ cd target/release
```
Now you move the game executable to the folder location where you want the game to be, here is how in linux:
```
$ mkdir ../../../raycaster
$ mv multithread_raycaster_game ../../../raycaster/
```
Now you go to the folder where the game is:
```
$ cd ../../../raycaster
```
Now you can Run it:
```
$ ./multithread_raycaster_game
```

## 🚀Usage

Currently as of this release when you run the program you will have a resolution selection with CLI input you can choose from a list of resolutions all which are limited to 16:9 aspect ratio currently, the following are the possible choices:
- 960x540
- 1024x576
- 1280x720
- 1366x768
- 1600x900
- 1920x1080
- 2560x1440
- 3200x1800
- 3840x2160

The second Prompt is the amount of threads for multithreding rendering, because of overhead 12 threads have the most perfomance on my computer atleast, anything above have less than 5% impact on perfomance, anything below and the perfomance degrades, here are the possible threads options:
- 1 Thread
- 2 Threads
- 4 Threads
- 6 Threads
- 8 Threads
- 12 Threads
- 16 Threads
- 24 Threads
- 32 Threads
- 64 Threads
- All Threads
- Default which is 80% or 4 if lower than 5, caps at 12

After you select your choices you are now inside the game, here are the keybinds:
- W: Walk forward
- S: Walk backwards
- A: Walk sideways left
- D: Walk sideways right
- Spacebar: Jump
- Left CTRL: Crouch
- Mouse X: rotate camera right or left
- Mouse Y: pitch up or down 
- Left ALT: Lock or Unlock mouse (toggle)
- ESC: Close game