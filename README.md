This is an attempt to create an undirected graph structure of NYC Housing Preservation & Development (HPD) records with the following properties:

1. Each individual name (and, optionally, corporation name) is a node.

2. Each address is a node.

3. A name node and address node are connected via an edge if at least one HPD registration contact contains both (i.e., if the name is associated with the address).

Portfolios are then inferred by finding strongly connected components within the graph.

This algorithm was created by Sam Rabiyah, Samara Trilling, and Atul Varma during the week of July 26, 2021.

## Quick start

A command-line tool called `hpd` provides various subcommands to analyze the graph. It uses CSV data directly from NYC's open data portal and does not require a database.

You will need [Rust][].

1. Download [NYC HPD Registration Contacts][hpd_reg_contacts] to `Registration_Contacts.csv`.

2. Download [NYC HPD Registrations][hpd_regs] to `Multiple_Dwelling_Registrations.csv`.

3. Run `cargo install --path .`.  (Alternatively, you can run the program directly via `cargo run --release --` if you don't want to install it.)

4. Run `hpd help`.

Note that the required CSV files must be in the current directory when you run the program.

[Rust]: https://www.rust-lang.org/
[hpd_regs]: https://data.cityofnewyork.us/Housing-Development/Multiple-Dwelling-Registrations/tesw-yqqr
[hpd_reg_contacts]: https://data.cityofnewyork.us/Housing-Development/Registration-Contacts/feu5-w2e2
