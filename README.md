`hpd` is a command-line tool that attempts to create an graph structure of NYC Housing Preservation & Development (HPD) records, along with various subcommands to analyze the graph.

It uses CSV data directly from NYC's open data portal and does not require a database.

## About the graph

The graph is undirected and has the following properties:

1. Each individual name (and, optionally, corporation name) is a node.

2. Each address is a node.

3. A name node and address node are connected via an edge if at least one HPD registration contact contains both (i.e., if the name is associated with the address).

Portfolios are then inferred by finding strongly connected components within the graph.

This algorithm was created by Sam Rabiyah, Samara Trilling, and Atul Varma during the week of July 26, 2021.

## Quick start

You will need [Rust][].

From the repository root, do the following:

1. Download [NYC HPD Registration Contacts][hpd_reg_contacts] to `Registration_Contacts.csv`.

2. Download [NYC HPD Registrations][hpd_regs] to `Multiple_Dwelling_Registrations.csv`.

3. Run `cargo install --path .`.  (Alternatively, you can run the program directly via `cargo run --release --` if you don't want to install it.)

4. Run `hpd help`.

Note that the required CSV files must be in the current directory when you run the program.

## Visualizing portfolios

### Using graphviz

Portfolios can be visualized with `hpd dot`, which outputs to the format that can be visualized by [graphviz][], but it can be difficult to display large portfolios using this method.

Here's an example of generating a portfolio as an SVG:

```
hpd dot "MOSES GUTMAN" | dot -Tsvg > portfolio.svg
```

### Using a web browser

An alternative is to use `hpd website` to export the largest portfolios as a static website.

To do this, you will also need [yarn][] and [nodejs][].

From the repository root, do the following:

1. Run `yarn`.

2. Run `yarn build`.

3. Run `hpd website`.

#### Developing the website

If you need to change the website's built JS bundle:

1. Run `yarn watch`.

2. Visit http://localhost:1234.

When you make any edits, you can reload your web browser to see the results.

[yarn]: https://yarnpkg.com/
[nodejs]: https://nodejs.org/en/
[graphviz]: https://graphviz.org/
[Rust]: https://www.rust-lang.org/
[hpd_regs]: https://data.cityofnewyork.us/Housing-Development/Multiple-Dwelling-Registrations/tesw-yqqr
[hpd_reg_contacts]: https://data.cityofnewyork.us/Housing-Development/Registration-Contacts/feu5-w2e2
