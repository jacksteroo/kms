//! Main entry point for the `tmkms` executable

use tmkms::application::APPLICATION;

/// Boot the `tmkms` application
fn main() {
    abscissa::boot(&APPLICATION);
}
