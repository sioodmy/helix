<div align="center">

<h1>
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="logo_dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="logo_light.svg">
  <img alt="Helix" height="128" src="logo_light.svg">
</picture>
</h1>
<img width="600" alt="Editor screnshot" src="https://github.com/user-attachments/assets/fdef5769-f89d-4e8a-9655-a4f12b769a6a" />
<img width="600" alt="Dashboard screenshot" src="https://github.com/user-attachments/assets/dc36bddd-e3e5-40ae-880d-41d3c86b90d1" />

</div>



My personal fork of Helix editor customized to my needs.

# Features

- [Alpha.nvim](https://github.com/goolord/alpha-nvim) like dashboard. The ascii art is hardcoded, because it is based on my girlfriend's tatto, so I will keep it like that.
- [Harpoon](https://github.com/ThePrimeagen/harpoon) implementation (credits: @ThePrimeagen)
- [nvim-colorizer](https://github.com/norcalli/nvim-colorizer.lua) implementation
- File browser (leader e)
- [lspkind.nvim](https://github.com/onsails/lspkind.nvim) implementation
- Smooth scrolling by default based on [smooth-scroll.hx](https://github.com/thomasschafer/smooth-scroll.hx)
- Rainbow brackets (credits: @SoraTenshi)
- Rainbow indent guides (credits: SoraTenshi)
- Foldable context (credits: @SoraTenshi)
- Rounded corners in built-in picker
- Themed bufferline inspired by [bufferline.nvim](https://github.com/akinsho/bufferline.nvim) (while keeping helix's default buffer behaviour)
- Themed statusline inspired by maaany nvim plugins

# Installation

Just use the Nix package

```nix
nix build github:sioodmy/helix
```

# Contributing

If you have any idea, just submit an issue and I might implement it. You can also submit a PR if you want to, but keep in mind this is my *personal* fork.
