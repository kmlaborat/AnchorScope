// Geometry calculator
// This module provides functions for calculating area and perimeter

fn calculate_area(width: f64, height: f64) -> f64 {
    // Calculate the area of a rectangle
    modified content
    width * height
}

fn calculate_perimeter(width: f64, height: f64) -> f64 {
    // Calculate the perimeter of a rectangle
    // Formula: 2 * (width + height)
    2.0 * (width + height)
}

fn main() {
    let w = 5.0;
    let h = 3.0;
    println!("Area: {}", calculate_area(w, h));
    println!("Perimeter: {}", calculate_perimeter(w, h));
}
