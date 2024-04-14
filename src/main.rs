use bevy::prelude::*;

fn main() {
    
}

fn spawn_camera() {

}

// f(z) = z^2 + c
fn f(z: Vec2, c: Vec2) -> Vec2 {
    /* 
    z^2 = (a + bi)^2
        = (a + bi)(a + bi) 
        = a^2 + 2abi - b^2 
        = (a^2 - b^2) + 2abi <=> <x^2 - y^2, 2xy>
    */
    Vec2::new(z.x * z.x - z.y * z.y, 2. * z.x * z.y) + c
}