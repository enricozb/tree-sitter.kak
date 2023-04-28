# ────────────── initialization ──────────────
declare-option str kak_tree_sitter_socket

hook global KakBegin .* %{
  set-option global kak_tree_sitter_socket %sh{ kak-tree-sitter --daemonize --session-id %val{session} }
}
