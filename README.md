# SaveIt

SaveIt is GUI tool for storing, editing and formatting sources for scientific texts.

Features
---

- Userfriendly GUI (using [egui](https://github.com/emilk/egui))
- Edit specific sources
- Format one or all sources and copy them to the clipboard

Updating
---

If you are updating from version `x.y.z` to a version with a higher `x` or `y` then your saved sources **will not show up** in the new version of the app.
This is because changes have been made to the database which makes these versions not compatible with each other.
If you want to continue working on your old sources **just launch the old version of the app** (the one you used previously).
If you want to get your sources from the old version to the new one, you will have to copy them manually.

Usage
---
On windows you won't be able to see any output if you run the program with any parameters but they will still be read by the program.
Running the program with any parameters in **not required** you can just doubleclick the executable. These options mostly just exist for debugging purposes.
```
Usage: SaveIt.exe [OPTIONS]

Options:
      --reset-config           Resets the config
      --reset-database         Resets the source database
      --verbosity <VERBOSITY>  Set logging verbosity level [default: info] [possible values: trace, debug, info, warn, error]
  -h, --help                   Print help
  -V, --version                Print version
```


Screenshots
---

![grafik](https://github.com/TgZ39/SaveIt/assets/71944761/5a1f05d9-eafb-4a1f-9998-b0b5e9c28f07)
![grafik](https://github.com/TgZ39/SaveIt/assets/71944761/10affc9e-f282-4c49-a3a7-336efacaa8db)
![grafik](https://github.com/TgZ39/SaveIt/assets/71944761/12826085-79fe-4d2b-bfb5-b7845d12193d)
![grafik](https://github.com/TgZ39/SaveIt/assets/71944761/07dfcc11-1121-4f32-a83a-6c06b4f54612)
