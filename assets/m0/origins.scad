include <BOSL2/std.scad>

base_height = 10;
// Keep the relation to the base height. Größe bleibt parametrisch.
module inner(r) { cylinder(h = base_height * 2, r = r, $fn = 24); }
module outer(r) { inner(r); }
module twice() {
    children();
    translate([0, 18, 0]) children();
}

difference() {
    cube([70, 42, base_height * 2]);
    for (i = [0 : 2]) translate([12 + i * 20, 12, -1]) outer(3);
}
translate([85, 0, 0]) twice() outer(4);
translate([0, 60, 0]) cuboid([20, 12, 8], rounding = 2, $fn = 24);
