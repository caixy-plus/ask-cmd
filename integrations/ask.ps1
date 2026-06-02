# PowerShell — `ask-cmd install --shell powershell`

function ask {
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Args)
    & ask-cmd @Args
}
