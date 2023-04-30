# ────────────── initialization ──────────────
declare-option str tree_sitter_socket
declare-option -hidden str tree_sitter_timestamp

# used for highlighting
declare-option -hidden range-specs tree_sitter_ranges
declare-option -hidden range-specs tree_sitter_ranges_spare

define-command -override tree-sitter-enable-buffer -docstring "start the tree-sitter server" %{
  evaluate-commands %sh{
    if [ -z "$kak_opt_tree_sitter_socket" ]; then
      printf \
        "set-option global tree_sitter_socket '%s'\n" \
        $(kak-tree-sitter --daemonize --session $kak_session)
    fi
  }

  # 1. send sync command to kak tree sitter to ask what buffer file to write to.
  # 2. setup hooks to write constantly to that file.

  tree-sitter-buffer-new

  hook -group tree-sitter buffer BufSetOption filetype=.* %{
    tree-sitter-buffer-set-language
  }

  hook -group tree-sitter buffer InsertIdle   .* tree-sitter-refresh
  hook -group tree-sitter buffer NormalIdle   .* tree-sitter-refresh
  hook -group tree-sitter buffer InsertChar   .* tree-sitter-refresh
  hook -group tree-sitter buffer InsertDelete .* tree-sitter-refresh
}

define-command -override tree-sitter-refresh %{
  evaluate-commands %sh{
    if [ "$kak_timestamp" != "$kak_opt_tree_sitter_timestamp" ]; then
      echo 'tree-sitter-buffer-save'
      echo 'tree-sitter-buffer-parse'
      echo 'tree-sitter-buffer-highlight'
      echo 'set-option buffer tree_sitter_timestamp %val{timestamp}'
    fi
  }
}


# ────────────── tree-sitter commands ──────────────
define-command -override tree-sitter-buffer-new -docstring "create a new buffer" %{
  tree-sitter-buffer-save
  tree-sitter-buffer-set-language
  tree-sitter-buffer-highlight

  try %{
    add-highlighter buffer/ ranges tree_sitter_ranges
  }
}


# ────────────── tree-sitter requests ──────────────
define-command -override -hidden tree-sitter-request -docstring "send request to tree-sitter" -params 1 %{
  nop %sh{ { echo "$1" | socat - UNIX-CONNECT:$kak_opt_tree_sitter_socket; } > /dev/null 2>&1 < /dev/null & }
}

define-command -override tree-sitter-reload %{
  tree-sitter-request "
    type   = 'reload_config'
  "
}

define-command -override -hidden tree-sitter-buffer-save  %{
  tree-sitter-request "
    type   = 'save_buffer'
    buffer = '%val{bufname}'
  "
}

define-command -override -hidden tree-sitter-buffer-set-language %{
  tree-sitter-request "
    type     = 'set_language'
    buffer   = '%val{bufname}'
    language = '%opt{filetype}'
  "
}

define-command -override -hidden tree-sitter-buffer-parse %{
  tree-sitter-request "
    type   = 'parse'
    buffer = '%val{bufname}'
  "
}

define-command -override -hidden tree-sitter-buffer-highlight %{
  tree-sitter-request "
    type   = 'highlight'
    buffer = '%val{bufname}'
  "
}
