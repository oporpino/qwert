use crate::ui::printer;

pub fn run() {
    printer::h1("qwert — dev environment manager");
    printer::blank();

    printer::h2("Machine setup");
    printer::command("use <tool> [--profile p]", "declare a tool and install it");
    printer::command("drop <tool>",   "remove tool from this machine's declaration");
    printer::command("apply",         "install/uninstall to match config.yml");
    printer::command("profile [name]", "show or set this machine's profile");
    printer::command("platform [platform]", "show or set this machine's platform (macos|debian|arch)");
    printer::blank();

    printer::h2("Information");
    printer::command("status [tool]", "show install status of declared tools");
    printer::command("list",          "list declared tools");
    printer::command("search <name>", "search recipes and yuiop (the package manager)");
    printer::blank();

    printer::h2("Maintenance");
    printer::command("upgrade [tool]",   "upgrade tools");
    printer::command("reinstall <tool>", "reinstall a tool");
    printer::command("update",           "update qwert and refresh recipes");
    printer::command("doctor",           "health check");
    printer::blank();

    printer::h2("Recipes");
    printer::command("recipes update",          "sync the default recipe catalog");
    printer::command("plugin add <url>",        "add a recipes repo (git clone)");
    printer::command("plugin remove <name>",    "remove a recipe plugin");
    printer::command("plugin list",             "list declared plugins");
    printer::command("plugin update",           "update all plugins");
    printer::blank();

    printer::h2("Config");
    printer::command("config edit",              "open qwert.yml in $EDITOR");
    printer::command("use script init    --path p", "add script to zsh init");
    printer::command("use script prepare --path p", "add script to zsh prepare");
    printer::blank();
}
