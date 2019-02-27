# clean-pii

A Rust executable that accomplishes the following:

- Combines person and purchase information from Ultimate Central data exports
- Cleans some Peronsally Identifiable Information (PII) from the data

This executable is only useful if you have the permissions to export these data from our website.

## Usage

`clean-pii` requires three things:

1. A csv file of people as exported from Ultimate Central
2. A directory of csv files of purchases as exported from Ultimate Central
3. A salt to encode people ids

It will write the cleaned records to standard output in csv format.
If `clean-pii` is installed in your `PATH`:

```bash
clean-pii people.csv purchases/ dont-be-so-salty > cleaned-data.csv
```

## Cleaning

The following cleaning operations are performed:

1. Many obvious fields are dropped on the floor (name, address ,etc)
2. Zip codes are transformed into Zip Code Tabulation Areas, as recommended [for HIPAA compliance](https://www.hhs.gov/hipaa/for-professionals/privacy/special-topics/de-identification/index.html#zip)
3. Person ids are encoded using a private salt, to prevent reverse URL lookups of people via their id
