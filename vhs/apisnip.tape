# This is a vhs script. See https://github.com/charmbracelet/vhs for more info.
# To run this script, install vhs and run `vhs ./vhs/apisnip.tape`
Output "target/apisnip.gif"
Set Theme "Catppuccin Frappe"
Set Width 1200
Set Height 768
Set WindowBar Colorful
Set BorderRadius 10
Hide
Type "cargo run -- https://raw.githubusercontent.com/swagger-api/swagger-petstore/refs/heads/master/src/main/resources/openapi.yaml"
Enter
Sleep 1s
Show
Sleep 2s
Set TypingSpeed 500ms
Down 5
Type " "
Type " "
Type " "
Up 1
Sleep 1s
Down 3
Sleep 1s
Type " "
Up 1
Sleep 1s
Type "w"
Set TypingSpeed 100ms
Type "cat apisnip.out.yaml | more"
Set TypingSpeed 1s
Enter
Type " "
Sleep 2s
Type " "
Sleep 2s
Type " "
Sleep 2s
Type " "
Sleep 2s
Type " "
Sleep 2s
Escape