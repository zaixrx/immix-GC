mod tl_alloc;
mod g_alloc;
mod block;

use g_alloc::GlobalAllocator;

#[derive(Debug, Clone)]
struct Player {
    name: String,
    score: usize, 
}

fn main() {
    let galloc = GlobalAllocator::<Player>::new();
    let player_ptr = galloc.alloc(size_of::<Player>()) as *mut Player;
    let player = unsafe { &mut (*player_ptr) };
    player.name = String::from("Hello, World!");
    player.score = 100;
    dbg!(player);
}
