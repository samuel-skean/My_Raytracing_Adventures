use vergen::EmitBuilder;
fn main() {
    EmitBuilder::builder()
    // I hope that this is what I really want - I'd like to be able to add
    // other untracked files without making the tree seem dirty for the
    // purpose of the output of the program:
        .git_dirty(false)
        .git_sha(false)
        .fail_on_error()
        .emit().unwrap();

    // TODO: Somehow checksum the whole src/ directory and put that into the json files created by skean_scene_gen.
    // Or, do something insane with pre-commit and post-commit hooks.
}