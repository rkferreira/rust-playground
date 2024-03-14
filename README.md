# rust-playground
My playground


## Vim configuration


```
git clone https://github.com/rust-lang/rust.vim ~/.vim/pack/plugins/start/rust.vim

curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
nvm install node

curl -fLo ~/.vim/autoload/plug.vim --create-dirs https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim

brew install hashicorp/tap/terraform-ls

```

```
#~/.vimrc

set expandtab
set tabstop=2
set shiftwidth=2

syntax enable
filetype plugin indent on


call plug#begin()
Plug 'neoclide/coc.nvim', {'branch': 'release'}
Plug 'tomasiser/vim-code-dark'
call plug#end()

```


```
vim

:PlugInstall
:CocInstall coc-rust-analyzer

```

### Yaml and Json validations

```
:CocInstall coc-json
:CocInstall coc-yaml
```


```
#~/.vim/coc-settings.json

{
  "rust-analyzer.inlayHints.typeHints.enable": false,
	"languageserver": {
		"terraform": {
			"command": "/opt/homebrew/bin/terraform-ls",
      "args": ["serve"],
			"filetypes": [
				"terraform",
				"tf"
			],
			"initializationOptions": {},
			"settings": {}
		}
	},
  "yaml.schemas": {
    "./schema/environment.json": [
      "domains/**/environment.*.yaml",
      "domains/**/environment.*.yml"
    ],
    "schema/domain.json": [
      "domains/**/domain.yaml",
      "domains/**/domain.yml"
    ]
  }
}

```
