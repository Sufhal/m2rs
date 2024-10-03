use m2rs::modules::conversion;

fn main() {
    conversion::msa::convert_msa();
    conversion::environment::convert_environments();
    conversion::property::convert_property();
    conversion::maps::convert_maps();
}