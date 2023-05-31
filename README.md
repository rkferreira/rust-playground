# rust-playground
My playground


## Vim configuration


```
git clone https://github.com/rust-lang/rust.vim ~/.vim/pack/plugins/start/rust.vim

curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
nvm install node

curl -fLo ~/.vim/autoload/plug.vim --create-dirs https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim

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
