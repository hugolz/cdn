#!/bin/sh
set -e

RUST_LOG="walrus=error"

mode=release # debug, release

out_dir=./front/out

echo Compiling
if [ "$mode" = release ]
then
  cargo build -p front --release --target=wasm32-unknown-unknown
else
  cargo build -p front --target=wasm32-unknown-unknown
fi

echo Bindgen
wasm-bindgen --target=web --out-dir=./target/wasm-bindgen/$mode ./target/wasm32-unknown-unknown/$mode/front.wasm --no-typescript

if ! [ -d $out_dir ]; then
    echo Creating ouput directory
    mkdir $out_dir
else
    echo Updating ouput directory
fi

cp ./target/wasm-bindgen/$mode/* $out_dir

cat << EOF > $out_dir/index.html
<!DOCTYPE html>
<html lang='en'>
  <head>
    <meta charset='utf-8'>
    <title>Yew • Counter</title>
    <link rel="stylesheet" type="text/css" href="./style.css">
    <script type='module'>
      import init from './front.js';
      init('front_bg.wasm');
    </script>
    
    <!--   <link rel='preload' href='./front_bg.wasm' as='fetch' type='application/wasm' crossorigin=''>
      <link rel='modulepreload' href='./front.js'> -->
  </head>
  <body>
  </body>
</html>
EOF

cat << EOF > $out_dir/style.css
button {
  background-color: #008f53; /* Green */
  border: 0;
  color: white;
  padding: 14px 14px;
  text-align: center;
  font-size: 16px;
  margin: 2px 2px;
  width: 100px;
}

.counter {
  color: #008f53;
  font-size: 48px;
  text-align: center;
}

.footer {
  text-align: center;
  font-size: 12px;
}

.panel {
  display: flex;
  justify-content: center;
}
EOF