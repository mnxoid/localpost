cargo build
target/debug/localpost
echo
target/debug/localpost stop
echo
target/debug/localpost upload example.txt
echo
target/debug/localpost stop --all
echo
target/debug/localpost list
echo
target/debug/localpost explore
echo
target/debug/localpost download example_key --output downloaded_example.txt
echo
target/debug/localpost download example_key
echo
target/debug/localpost stop example_key
