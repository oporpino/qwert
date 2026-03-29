use crate::ui::printer;

pub fn run() {
    printer::h1("qwert — dev environment manager");
    printer::blank();

    printer::h2("Machine setup");
    printer::command("use <tool>",    "declare a tool for this machine and install it");
    printer::command("drop <tool>",   "remove tool from this machine's declaration");
    printer::command("apply",         "install/uninstall to match qwert.yml");
    printer::blank();

    printer::h2("Information");
    printer::command("status [tool]", "show install status of declared tools");
    printer::command("list",          "list declared tools");
    printer::command("search <name>", "search recipes and brew");
    printer::blank();

    printer::h2("Maintenance");
    printer::command("upgrade [tool]",   "upgrade tools");
    printer::command("reinstall <tool>", "reinstall a tool");
    printer::command("update",           "update qwert and refresh recipes");
    printer::command("doctor",           "health check");
    printer::blank();

    printer::h2("Config");
    printer::command("config edit",              "open qwert.yml in $EDITOR");
    printer::command("use script init --path p", "add script to zsh init");
    printer::command("use script end  --path p", "add script to zsh end");
    printer::blank();
}
