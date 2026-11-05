# Close to Tray（Becky! 2 プラグイン）

Becky! 2 のメインウィンドウを，閉じる代わりにタスクトレイへ入れるプラグイン．

## 機能

| 操作 | 動作 |
|---|---|
| メインウィンドウの × ボタン，Alt+F4 | 終了せず，タスクトレイに入る |
| 表示中にタスクトレイアイコンを左クリック | タスクトレイに入る |
| タスクトレイに入っているときにアイコンを左クリック | 表示する（Becky! 本体の動作） |
| メニューの「終了」，タスクトレイアイコンの右クリックメニューの「終了」 | 終了する |

× ボタンと Alt+F4 は最小化に置き換える．最小化したウィンドウをタスクトレイに入れるのは Becky! 本体なので，Becky! の設定で「最小化時はタスクトレイ（通知領域）のみに表示」を有効にしておく必要がある．

## 動作環境

- Becky! 2（32bit 版）．2.83.0.8 で動作を確認した．
- Windows

タスクトレイアイコンのクリックは，Becky! がアイコンに設定した通知メッセージ（`0x0BDA`，アイコン ID `0x80`）を見て判定している．この値は Becky! の公開 API ではなく，2.83.0.8 で観測したもの．Becky! の版によって変わった場合は，下記の `trace-messages` で調べ直して `src/main_window.rs` の定数を直す．

## インストール

1. `becky2_tray_plugin.dll` を次のどちらかのフォルダに置く．
   - `[Becky! のプログラムフォルダ]\PlugIns\`
   - `[Becky! のデータフォルダ]\PlugIns\`
2. Becky! を起動する．

DLL を置き換えるときは，先に Becky! を終了しておく．Becky! が DLL を読み込んでいる間は上書きできない．

## ビルド

B2.exe は 32bit なので，DLL も 32bit（`i686-pc-windows-msvc`）でビルドする．`.cargo/config.toml` でこのターゲットを既定にしてある．

必要なもの:

- Rust（rustup）と MSVC のビルドツール
- 32bit のターゲット: `rustup target add i686-pc-windows-msvc`

```sh
cargo build --release
```

`target/i686-pc-windows-msvc/release/becky2_tray_plugin.dll` ができる．

Becky! は呼び出し規約の装飾が付かない名前（`BKC_OnStart` など）で関数を探す．そのため，`build.rs` から `becky2_tray_plugin.def` をリンカに渡して，export する名前を指定している．

## メッセージの記録（trace-messages）

Becky! が受け取るウィンドウメッセージを調べるための機能．既定のビルドにはログを書くコードが入らないので，通常の利用ではログを取らない．使うときは，feature を指定して調査用の DLL をビルドする．

```sh
cargo build --release --features trace-messages --target-dir target/trace
```

この DLL を使うと，DLL と同じフォルダの `becky2_tray_plugin.log` に次のものを書き出す．

- Becky! の UI スレッドのトップレベルウィンドウ（クラス名，タイトル）
- それらのウィンドウが受け取った `WM_USER` 以上のメッセージ，`WM_SYSCOMMAND`，`WM_CLOSE`
- `BKC_OnMenuInit` に渡されたメニューの項目

マウスをタスクトレイアイコンの上で動かすだけでもログが増えるので，調べ終わったら既定のビルドに戻す．

## ライセンス

[MIT](LICENSE)

Becky! Plug-In SDK（RimArts）の宣言を参考にしているが，SDK のファイルそのものは含んでいない．
