using System.Collections.Generic;

namespace Domain.Configuration.Models;

public class CodeGenConfig
{
    public ProjectConfig Project { get; set; } = new();
    public BackendConfig Backend { get; set; } = new();
    public UiConfig Ui { get; set; } = new();
    public List<ModuleConfig> Modules { get; set; } = new();
}

public class ProjectConfig
{
    public string Name { get; set; } = string.Empty;
    public string Description { get; set; } = string.Empty;
}

public class BackendConfig
{
    public string Type { get; set; } = "cleanArch";
    public string ArchStyle { get; set; } = "clean-cqrs"; // "flat" | "layered" | "clean-cqrs"
    public string Pattern { get; set; } = "separated";    // "basic" | "separated"
    public string HostType { get; set; } = "api";         // "api" | "mvc" | "console"
    public string Paradigm { get; set; } = "oop";
    public string Style { get => Paradigm; set => Paradigm = value; }
    public string Dispatcher { get; set; } = "native";
    public string ApiStyle { get; set; } = "controller";

    public bool IsFlat => string.Equals(ArchStyle, "flat", System.StringComparison.OrdinalIgnoreCase);
    public bool IsLayered => string.Equals(ArchStyle, "layered", System.StringComparison.OrdinalIgnoreCase);
    public bool IsCleanCqrs => string.Equals(ArchStyle, "clean-cqrs", System.StringComparison.OrdinalIgnoreCase) || (!IsFlat && !IsLayered);

    public bool IsBasicPattern => string.Equals(Pattern, "basic", System.StringComparison.OrdinalIgnoreCase);
    public bool IsSeparatedPattern => !IsBasicPattern;

    public bool IsMvcHost => string.Equals(HostType, "mvc", System.StringComparison.OrdinalIgnoreCase);
    public bool IsConsoleHost => string.Equals(HostType, "console", System.StringComparison.OrdinalIgnoreCase);
    public bool IsApiHost => !IsMvcHost && !IsConsoleHost;

    public bool IsControllerless => string.Equals(ApiStyle, "minimal", System.StringComparison.OrdinalIgnoreCase) || string.Equals(ApiStyle, "controllerless", System.StringComparison.OrdinalIgnoreCase);
    public bool IsMediatR => string.Equals(Dispatcher, "mediatr", System.StringComparison.OrdinalIgnoreCase);
    public bool IsMediatr => IsMediatR;
    public bool Secured { get; set; } = false;
    public bool SecuredCommands { get; set; } = false;
    public bool SecuredQueries { get; set; } = false;
    public bool IsSecuredCommands => Secured || SecuredCommands;
    public bool IsSecuredQueries => Secured || SecuredQueries;
    public string Path { get; set; } = string.Empty;
    public string DbContext { get; set; } = string.Empty;
    public string Namespace { get; set; } = string.Empty;
    public BackendLayersConfig Layers { get; set; } = new();
    public BackendStructureConfig Structure { get; set; } = new();
}

public class BackendLayersConfig
{
    public string Domain { get; set; } = "Domain";
    public string Application { get; set; } = "Application";
    public string Persistence { get; set; } = "Persistence";
    public string Infrastructure { get; set; } = "Infrastructure";
    public string Api { get; set; } = "API";
    public string Host { get; set; } = string.Empty;
    public string Core { get; set; } = "Core";

    // Backward compatibility for existing configurations using WebApi
    public string WebApi
    {
        get => Api;
        set => Api = value;
    }
}

public class BackendStructureConfig
{
    public string Mode { get; set; } = "feature"; // "feature" | "layer"
    public string FolderName { get; set; } = "Features";
}

public class UiConfig
{
    public string Type { get; set; } = "angular";
    public string Path { get; set; } = string.Empty;
    public string BaseUrl { get; set; } = string.Empty;
    public string HttpClient { get; set; } = "fetch"; // "fetch" | "axios"
    public UiStructureConfig Structure { get; set; } = new();
    public UiDirsConfig Dirs { get; set; } = new();
    public UiStateConfig State { get; set; } = new();
}

public class UiStructureConfig
{
    public string Mode { get; set; } = "direct"; // "direct" | "grouped"
    public string Prefix { get; set; } = string.Empty;
}

public class UiDirsConfig
{
    public string Components { get; set; } = "components";
    public string Services { get; set; } = "services";
    public string Store { get; set; } = "store";
    public string Models { get; set; } = "models";
    public string Pages { get; set; } = "pages";
}

public class UiStateConfig
{
    public string Strategy { get; set; } = "signals"; // "signals" | "ngrx"
}

public class ModuleConfig
{
    public string Name { get; set; } = string.Empty;
    public string IdType { get; set; } = "int";
    public bool GenerateUi { get; set; } = true;
    public ModuleBackendConfig Backend { get; set; } = new();
    public ModuleUiConfig Ui { get; set; } = new();
    public List<PropertyConfig> Properties { get; set; } = new();
    public List<SubModuleConfig> SubModules { get; set; } = new();
}

public class ModuleBackendConfig
{
    public bool Caching { get; set; } = true;
    public bool Logging { get; set; } = true;
    public bool Transaction { get; set; } = true;
    public bool Secured { get; set; } = true;
    public bool SecuredCommands { get; set; } = false;
    public bool SecuredQueries { get; set; } = false;
    public bool IsSecuredCommands => Secured || SecuredCommands;
    public bool IsSecuredQueries => Secured || SecuredQueries;
}

public class ModuleUiConfig
{
    public bool ListPage { get; set; } = true;
    public bool DetailPage { get; set; } = true;
    public bool FormPage { get; set; } = true;
}

public class PropertyConfig
{
    public string Name { get; set; } = string.Empty;
    public string Type { get; set; } = "string";
    public bool Required { get; set; } = false;
}

public class SubModuleConfig
{
    public string Name { get; set; } = string.Empty;
    public string IdType { get; set; } = "int";
    public bool GenerateUi { get; set; } = true;
    public string Parent { get; set; } = string.Empty;
    public List<PropertyConfig> Properties { get; set; } = new();
}
