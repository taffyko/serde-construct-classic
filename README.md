A Rust library using the Serde serialization framework to support Construct Classic's serialization data formats,
plus a CLI to easily convert to and from JSON.

Currently supports Construct Classic's "HashTable" data format.

## CLI Usage

Convert HashTable file to JSON file:  
`cstc_json tabletojson ./file.lvl ./file.json`

Convert JSON file to HashTable file:  
`cstc_json jsontotable ./file.json ./file.lvl`