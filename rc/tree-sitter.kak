# ────────────── initialization ──────────────
declare-option str tree_sitter_socket

define-command -override tree-sitter-enable -docstring "start the tree-sitter server" %{
  evaluate-commands %sh{
    if [ -z "$kak_opt_tree_sitter_socket" ]; then
      printf \
        "set-option global tree_sitter_socket '%s'\n" \
        $(kak-tree-sitter --daemonize --session-id $kak_session)
    fi
  }

  tree-sitter-new-buffer

  hook -group tree-sitter buffer BufSetOption filetype=.* %{
    tree-sitter-set-language
  }

  hook -group tree-sitter buffer InsertIdle '' %{
    tree-sitter-save
    tree-sitter-parse
  }

  hook -group tree-sitter buffer ModeChange 'pop:insert:normal' %{
    tree-sitter-save
    tree-sitter-parse
  }
}


# ────────────── tree-sitter commands ──────────────
define-command -override tree-sitter-new-buffer -docstring "create a new buffer" %{
  tree-sitter-save
  tree-sitter-set-language
}


# ────────────── tree-sitter requests ──────────────
define-command -override -hidden tree-sitter-request -docstring "send request to tree-sitter" -params 1 %{
  evaluate-commands %sh{ echo "$1" | socat - $kak_opt_tree_sitter_socket }
}

define-command -override -hidden tree-sitter-save -docstring "save buffer" %{
  tree-sitter-request "
    type   = 'save_buffer'
    buffer = '%val{bufname}'
  "
}

define-command -override -hidden tree-sitter-set-language -docstring "set language" %{
  tree-sitter-request "
    type     = 'set_language'
    buffer   = '%val{bufname}'
    language = '%opt{filetype}'
  "
}

define-command -override -hidden tree-sitter-parse -docstring "set language" %{
  tree-sitter-request "
    type   = 'parse'
    buffer = '%val{bufname}'
  "
}
