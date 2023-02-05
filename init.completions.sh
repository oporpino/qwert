#/usr/bin/env bash
_completions()
{
    local cur prev

    cur=${COMP_WORDS[COMP_CWORD]}
    prev=${COMP_WORDS[COMP_CWORD-1]}

    case ${COMP_CWORD} in
        1)
            COMPREPLY=($(compgen -W "setup status" -- ${cur}))
            ;;
        2)
            case ${prev} in
                setup)
                    COMPREPLY=($(compgen -W "macos linux" -- ${cur}))
                    ;;
                show)
                    COMPREPLY=($(compgen -W "all homebrew lvim" -- ${cur}))
                    ;;
            esac
            ;;
        *)
            COMPREPLY=()
            ;;
    esac
}

complete -F _completions qwert
