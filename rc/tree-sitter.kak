# ────────────── initialization ──────────────
declare-option str tree_sitter_config
declare-option str tree_sitter_fifo_req
declare-option str tree_sitter_fifo_buf

# timestamp to debounce buffer change hooks.
declare-option -hidden str tree_sitter_timestamp

# used for highlighting
declare-option -hidden range-specs tree_sitter_ranges
declare-option -hidden range-specs tree_sitter_ranges_spare

define-command tree-sitter-enable-buffer -docstring "start the tree-sitter server" %{
  evaluate-commands %sh{
    if [ -z "$kak_opt_tree_sitter_fifo_req" ]; then
      if [ -n "$kak_opt_tree_sitter_config" ]; then
        config="--config $kak_opt_tree_sitter_config"
      fi

      # this will print out commands to set the fifo_req and fifo_buf options
      ./target/release/kak-sitter --daemonize --session $kak_session $config
    fi
  }

  # 0. remove any extant hooks
  remove-hooks buffer tree-sitter

  # 1. send command to server setting langauge and buffer name.
  tree-sitter-new-buffer

  # 2. setup hooks to write constantly to that file.
  hook -group tree-sitter buffer InsertIdle   .* tree-sitter-refresh
  hook -group tree-sitter buffer NormalIdle   .* tree-sitter-refresh
  hook -group tree-sitter buffer BufReload    .* tree-sitter-refresh

  hook -group tree-sitter buffer BufSetOption filetype=.* %{
    tree-sitter-set-language
    tree-sitter-refresh
  }

  # 3. add highlighter
  try %{
    add-highlighter buffer/ ranges tree_sitter_ranges
  }
}

define-command tree-sitter-refresh %{
  evaluate-commands -no-hooks %sh{
    if [ "$kak_timestamp" != "$kak_opt_tree_sitter_timestamp" ]; then
      echo 'tree-sitter-parse-buffer'
      echo 'tree-sitter-highlight-buffer'
      echo 'set-option buffer tree_sitter_timestamp %val{timestamp}'
    fi
  }
}


# ────────────── tree-sitter requests ──────────────
define-command -hidden tree-sitter-request -docstring "send request to tree-sitter" -params 1 %{
  nop %sh{
    echo "$1" > $kak_opt_tree_sitter_fifo_req
  }
}

define-command -hidden tree-sitter-write-buffer -docstring "send buffer contents to tree-sitter" %{
  write %opt{tree_sitter_fifo_buf}
}

define-command tree-sitter-reload-config %{
  tree-sitter-request "
    type   = 'reload_config'
    config = '%opt{tree_sitter_config}'
  "
}

define-command tree-sitter-new-buffer %{
  tree-sitter-request "
    type     = 'new_buffer'
    buffer   = '%val{bufname}'
    language = '%opt{filetype}'
  "
}

define-command tree-sitter-set-language %{
  tree-sitter-request "
    type     = 'set_language'
    buffer   = '%val{bufname}'
    language = '%opt{filetype}'
  "
}

define-command tree-sitter-parse-buffer %{
  tree-sitter-request "
    type      = 'parse_buffer'
    buffer    = '%val{bufname}'
  "

  tree-sitter-write-buffer
}

define-command tree-sitter-highlight-buffer %{
  tree-sitter-request "
    type      = 'highlight'
    buffer    = '%val{bufname}'
  "
}
