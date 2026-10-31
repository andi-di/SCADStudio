include <deeper/value.scad>
thickness = height_value;
module resource_shape() {
    linear_extrude(height = thickness) import("shape.svg");
}
