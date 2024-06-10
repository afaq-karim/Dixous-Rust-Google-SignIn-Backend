# Dixous-Rust-Google-SignIn-Backend

Backend server for Dixous Rust framework, facilitating Google Sign-In. Powered by Rocket, a fast, secure, and flexible Rust framework.

## Setup

1. **Install Rust:**
   Refer to the [official Rust website](https://www.rust-lang.org/tools/install) for instructions on installing Rust and Cargo, Rust's package manager.

2. **Install Dependencies:**
   Navigate to the project directory and run:
   ```bash
   cargo build
Usage
Run the Server:
Start the server by executing:

```bash
Copy code
cargo run
Endpoints:

/auth/google: Endpoint for Google Sign-In.
/auth/google/callback?<code>: Endpoint to retrieve user information.
Configuration
Environment Variables:
ROCKET_PORT: Port on which the server runs. The default is 8000.
```

Contributing
Contributions are welcome! Fork the repository, make your changes, and submit a pull request. Ensure to follow the code of conduct.

License
This project is licensed under the MIT License. See the LICENSE file for details.
