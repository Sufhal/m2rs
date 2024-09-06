use m2rs::modules::conversion;

fn main() {
    conversion::environment::convert_environments();
    conversion::property::convert_property();
    conversion::maps::convert_maps();
}