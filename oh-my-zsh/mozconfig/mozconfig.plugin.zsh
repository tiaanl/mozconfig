MOZCONFIG_LABEL_COLOR=blue
MOZCONFIG_CONFIG_COLOR=red

function prompt_mozconfig() {
    CONFIG=$(mozconfig 2&>/dev/null)
    if [ ! -z $CONFIG ];
    then
        echo "%{$fg_bold[$MOZCONFIG_LABEL_COLOR]%}mozconfig:(%{$fg_bold[$MOZCONFIG_CONFIG_COLOR]%}$CONFIG%{$fg_bold[$MOZCONFIG_LABEL_COLOR]%})%{$reset_color%} "
    fi
}

. $ZSH/plugins/mozconfig/mozconfig-completions.bash
