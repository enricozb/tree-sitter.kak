# ────────────── initialization ──────────────
declare-option str tree_sitter_socket

# used for highlighting
declare-option -hidden range-specs tree_sitter_ranges
declare-option -hidden range-specs tree_sitter_ranges_spare

define-command -override tree-sitter-enable -docstring "start the tree-sitter server" %{
  evaluate-commands %sh{
    if [ -z "$kak_opt_tree_sitter_socket" ]; then
      printf \
        "set-option global tree_sitter_socket '%s'\n" \
        $(kak-tree-sitter --daemonize --session $kak_session)
    fi
  }

  tree-sitter-buffer-new

  hook -group tree-sitter buffer BufSetOption filetype=.* %{
    tree-sitter-buffer-set-language
  }

  hook -group tree-sitter buffer InsertIdle '' %{
    tree-sitter-buffer-save
    tree-sitter-buffer-parse
    tree-sitter-buffer-highlight
  }

  hook -group tree-sitter buffer InsertChar .* %{
    tree-sitter-buffer-save
    tree-sitter-buffer-parse
    tree-sitter-buffer-highlight
  }


  hook -group tree-sitter buffer ModeChange 'pop:insert:normal' %{
    tree-sitter-buffer-save
    tree-sitter-buffer-parse
    tree-sitter-buffer-highlight
  }
}


# ────────────── tree-sitter commands ──────────────
define-command -override tree-sitter-buffer-new -docstring "create a new buffer" %{
  tree-sitter-buffer-save
  tree-sitter-buffer-set-language

  try %{
    add-highlighter buffer/ ranges tree_sitter_ranges
  }
}


# ────────────── tree-sitter requests ──────────────
define-command -override -hidden tree-sitter-buffer-request -docstring "send request to tree-sitter" -params 1 %{
  evaluate-commands %sh{ echo "$1" | socat - $kak_opt_tree_sitter_socket }
}

define-command -override -hidden tree-sitter-reload %{
  tree-sitter-buffer-request "
    type   = 'reload_config'
  "
}

define-command -override -hidden tree-sitter-buffer-save  %{
  tree-sitter-buffer-request "
    type   = 'save_buffer'
    buffer = '%val{bufname}'
  "
}

define-command -override -hidden tree-sitter-buffer-set-language %{
  tree-sitter-buffer-request "
    type     = 'set_language'
    buffer   = '%val{bufname}'
    language = '%opt{filetype}'
  "
}

define-command -override -hidden tree-sitter-buffer-parse %{
  tree-sitter-buffer-request "
    type   = 'parse'
    buffer = '%val{bufname}'
  "
}

define-command -override -hidden tree-sitter-buffer-highlight %{
  tree-sitter-buffer-request "
    type   = 'highlight'
    buffer = '%val{bufname}'
  "
}
