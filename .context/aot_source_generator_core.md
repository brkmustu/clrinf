### Mimari Senaryo (E-Ticaret Karşılığı)

* **Standart Parametre Tabloları (AOT CRUD):** `Category` (Kategori), `Brand` (Marka), `Currency` (Para Birimi), `Country` (Ülke), `Warehouse` (Depo) gibi yüzlerce standart CRUD tablosu.
* **Özel İş Kuralı (Hook) Senaryosu:** *"Bir **Kategori (`Category`)** silinirken, eğer o kategoriye bağlı aktif ürünler (`IProductCatalogService`) varsa silme işlemi engellensin."*
* **Kısmi Çıkarma (`eject`) Senaryosu:** *"Kategorinin `Update` işlemi sırasında SEO URL (Slug) geçmişi ve Elasticsearch indeksleri güncelleneceği için sadece `Update` operasyonu AOT Generator'dan fiziksel dosyaya çıkarılsın."*

---

## Faz 1: Çekirdek Kontratlar (`Commerce.Core`)

Hem ana e-ticaret uygulamasının hem de Source Generator'ın kullandığı temel attribute ve iş kuralı (hook) arayüzleri:

### 1. `GenerateCrudEndpointsAttribute.cs`

```csharp
namespace Commerce.Core.Attributes;

[Flags]
public enum CrudOperations
{
    None = 0,
    Create = 1,
    Update = 2,
    Delete = 4,
    GetById = 8,
    GetList = 16,
    All = Create | Update | Delete | GetById | GetList
}

[AttributeUsage(AttributeTargets.Class, Inherited = false, AllowMultiple = false)]
public sealed class GenerateCrudEndpointsAttribute : Attribute
{
    public CrudOperations Operations { get; set; } = CrudOperations.All;
    public string? RoutePrefix { get; set; }
}

```

### 2. `IBusinessRules.cs` (AOT & Test Dostu Kancalar)

```csharp
namespace Commerce.Core.Abstractions;

public readonly record struct OperationResult(bool IsSuccess, string? ErrorMessage = null)
{
    public static OperationResult Success() => new(true);
    public static OperationResult Failure(string error) => new(false, error);
}

public interface IBeforeCreateRule<in TEntity> where TEntity : class
{
    Task<OperationResult> OnBeforeCreateAsync(TEntity entity, CancellationToken ct = default);
}

public interface IBeforeUpdateRule<in TEntity> where TEntity : class
{
    Task<OperationResult> OnBeforeUpdateAsync(TEntity entity, CancellationToken ct = default);
}

public interface IBeforeDeleteRule<in TEntity> where TEntity : class
{
    Task<OperationResult> OnBeforeDeleteAsync(TEntity entity, CancellationToken ct = default);
}

```

---

## Faz 2: Roslyn Source Generator Motoru (`Commerce.CodeGen.Crud`)

`[GenerateCrudEndpoints]` işaretli e-ticaret varlıklarını (`Category`, `Brand` vb.) tarayan, geliştiricinin elle yazdığı (`eject` ettiği) sınıfları otomatik atlayan (**Smart Override**) ve kanca destekli Handler'ları bellekte üreten `IIncrementalGenerator` kodu:

### `CrudIncrementalGenerator.cs`

```csharp
using System.Collections.Immutable;
using System.Text;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Text;

namespace Commerce.CodeGen.Crud;

[Generator(LanguageNames.CSharp)]
public sealed class CrudIncrementalGenerator : IIncrementalGenerator
{
    private const string AttributeFullName = "Commerce.Core.Attributes.GenerateCrudEndpointsAttribute";

    public void Initialize(IncrementalGeneratorInitializationContext context)
    {
        // 1. [GenerateCrudEndpoints] olan Entity sınıflarını yakala
        IncrementalValuesProvider<EntityGenerationModel?> entityDeclarations = context.SyntaxProvider
            .ForAttributeWithMetadataName(
                fullyQualifiedMetadataName: AttributeFullName,
                predicate: static (node, _) => node is ClassDeclarationSyntax,
                transform: static (ctx, ct) => GetEntityModel(ctx, ct))
            .Where(static m => m is not null);

        // 2. Compilation ile birleştir (Smart Override kontrolü için)
        IncrementalValueProvider<(Compilation, ImmutableArray<EntityGenerationModel?>)> compilationAndEntities =
            context.CompilationProvider.Combine(entityDeclarations.Collect());

        // 3. Kodu derleme anında (AOT) üret
        context.RegisterSourceOutput(compilationAndEntities, static (spc, source) => Execute(source.Item1, source.Item2, spc));
    }

    private static EntityGenerationModel? GetEntityModel(GeneratorAttributeSyntaxContext context, CancellationToken ct)
    {
        if (context.TargetSymbol is not INamedTypeSymbol entitySymbol)
            return null;

        var entityName = entitySymbol.Name;
        var entityNamespace = entitySymbol.ContainingNamespace.ToDisplayString();

        string routePrefix = $"/api/{entityName.ToLowerInvariant()}s";
        var attr = context.Attributes[0];
        foreach (var namedArg in attr.NamedArguments)
        {
            if (namedArg.Key == "RoutePrefix" && namedArg.Value.Value is string customRoute)
                routePrefix = customRoute;
        }

        var properties = entitySymbol.GetMembers()
            .OfType<IPropertySymbol>()
            .Where(p => p.DeclaredAccessibility == Accessibility.Public && !p.IsStatic)
            .Select(p => new PropertyModel(p.Name, p.Type.ToDisplayString()))
            .ToImmutableArray();

        return new EntityGenerationModel(entityName, entityNamespace, routePrefix, properties);
    }

    private static void Execute(
        Compilation compilation,
        ImmutableArray<EntityGenerationModel?> entities,
        SourceProductionContext context)
    {
        if (entities.IsDefaultOrEmpty) return;

        var validEntities = new List<EntityGenerationModel>();

        foreach (var entity in entities.Where(e => e is not null))
        {
            var model = entity!;
            validEntities.Add(model);

            // SMART OVERRIDE KONTROLÜ:
            // Örneğin geliştirici "DeleteCategoryCommandHandler" sınıfını fiziksel olarak yazmış mı?
            var expectedHandlerFullName = $"Commerce.Application.Features.{model.Name}s.Delete{model.Name}CommandHandler";
            bool isManuallyOverridden = compilation.GetTypeByMetadataName(expectedHandlerFullName) is not null;

            if (!isManuallyOverridden)
            {
                var handlerCode = EmitDeleteFeature(model);
                context.AddSource($"{model.Name}_DeleteFeature.g.cs", SourceText.From(handlerCode, Encoding.UTF8));
            }
        }

        // Reflection kullanmadan tüm üretilen sınıfları DI'a kaydeden AOT metodu
        var diRegistrationCode = EmitAotDependencyInjection(validEntities);
        context.AddSource("GeneratedCrudServiceCollectionExtensions.g.cs", SourceText.From(diRegistrationCode, Encoding.UTF8));
    }

    private static string EmitDeleteFeature(EntityGenerationModel entity)
    {
        return $$"""
        // <auto-generated />
        #nullable enable
        using System.Collections.Generic;
        using System.Threading;
        using System.Threading.Tasks;
        using Commerce.Core.Abstractions;
        using {{entity.Namespace}};

        namespace Commerce.Application.Features.{{entity.Name}}s;

        public partial record Delete{{entity.Name}}Command(int Id);

        public sealed partial class Delete{{entity.Name}}CommandHandler
        {
            private readonly ICommerceDbContext _dbContext;
            private readonly IEnumerable<IBeforeDeleteRule<{{entity.Name}}>> _beforeDeleteRules;

            public Delete{{entity.Name}}CommandHandler(
                ICommerceDbContext dbContext,
                IEnumerable<IBeforeDeleteRule<{{entity.Name}}>> beforeDeleteRules)
            {
                _dbContext = dbContext;
                _beforeDeleteRules = beforeDeleteRules;
            }

            public async Task<OperationResult> HandleAsync(Delete{{entity.Name}}Command command, CancellationToken ct = default)
            {
                var entity = await _dbContext.Set<{{entity.Name}}>().FindAsync(new object[] { command.Id }, ct);
                if (entity is null)
                    return OperationResult.Failure("{{entity.Name}} kaydı bulunamadı.");

                // İŞ KURALI KANCALARI (Örn: Kategoriye bağlı aktif ürün var mı?)
                foreach (var rule in _beforeDeleteRules)
                {
                    var ruleResult = await rule.OnBeforeDeleteAsync(entity, ct);
                    if (!ruleResult.IsSuccess)
                        return ruleResult;
                }

                _dbContext.Set<{{entity.Name}}>().Remove(entity);
                await _dbContext.SaveChangesAsync(ct);

                return OperationResult.Success();
            }
        }
        """;
    }

    private static string EmitAotDependencyInjection(List<EntityGenerationModel> entities)
    {
        var sb = new StringBuilder();
        sb.AppendLine("""
        // <auto-generated />
        using Microsoft.Extensions.DependencyInjection;

        namespace Commerce.Application.Generated;

        public static class GeneratedCrudServiceCollectionExtensions
        {
            public static IServiceCollection AddGeneratedCrudHandlers(this IServiceCollection services)
            {
        """);

        foreach (var entity in entities)
        {
            sb.AppendLine($"        services.AddScoped<Commerce.Application.Features.{entity.Name}s.Delete{entity.Name}CommandHandler>();");
        }

        sb.AppendLine("""
                return services;
            }
        }
        """);

        return sb.ToString();
    }
}

internal sealed record EntityGenerationModel(
    string Name,
    string Namespace,
    string RoutePrefix,
    ImmutableArray<PropertyModel> Properties);

internal sealed record PropertyModel(string Name, string TypeFullName);

```

---

## Faz 3: `ArchGen.Cli` Konsol Uygulaması ve Şablon Motoru

E-ticaret projesinde yeni bir iş kuralı (`add-rule`) ekleyen veya karmaşıklaşan bir operasyonu fiziksel dosyaya çıkaran (`eject`) CLI aracının kodları:

### 1. `Program.cs` (CLI Komut Tanımları)

```csharp
using System.CommandLine;
using ArchGen.Cli.Commands;

var rootCommand = new RootCommand("E-Commerce Architecture AOT & Hook Code Generator CLI (archgen)");

var entityArg = new Argument<string>("entity", "Hedef Entity adı (Örn: Category, Brand, Warehouse)");
var pathOption = new Option<string>("--project-path", () => Directory.GetCurrentDirectory(), "Solution kök dizini");

// 1. KOMUT: archgen add-rule Category --on BeforeDelete --name ActiveProductsCheck
var hookOption = new Option<string>("--on", "Kanca tipi: BeforeCreate, BeforeUpdate, BeforeDelete") { IsRequired = true };
var nameOption = new Option<string>("--name", "İş kuralının adı (Örn: ActiveProductsCheck)") { IsRequired = true };

var addRuleCommand = new Command("add-rule", "AOT yapısını bozmadan Entity için izole bir iş kuralı (Hook) ve Unit Test üretir.")
{
    entityArg, hookOption, nameOption, pathOption
};

addRuleCommand.SetHandler(async (entity, hook, ruleName, projectPath) =>
{
    await AddRuleCommandHandler.ExecuteAsync(entity, hook, ruleName, projectPath);
}, entityArg, hookOption, nameOption, pathOption);

// 2. KOMUT: archgen eject Category --operation Update
var ejectOperationOption = new Option<string>("--operation", "Fiziksel dosyaya çıkarılacak operasyon: Create, Update, Delete, GetById, GetList") { IsRequired = true };

var ejectCommand = new Command("eject", "Belirtilen CRUD operasyonunu AOT Generator'dan çıkarır ve fiziksel C# dosyası olarak diske yazar.")
{
    entityArg, ejectOperationOption, pathOption
};

ejectCommand.SetHandler(async (entity, operation, projectPath) =>
{
    await EjectCommandHandler.ExecuteAsync(entity, operation, projectPath);
}, entityArg, ejectOperationOption, pathOption);

rootCommand.AddCommand(addRuleCommand);
rootCommand.AddCommand(ejectCommand);

return await rootCommand.InvokeAsync(args);

```

### 2. `TemplateEngine.cs` (E-Ticaret Şablon Motoru)

```csharp
namespace ArchGen.Cli.Templates;

public static class TemplateEngine
{
    public static string GenerateBusinessRule(string entityName, string hookType, string ruleName)
    {
        var interfaceName = $"I{hookType}Rule<{entityName}>";
        var methodName = $"On{hookType}Async";
        var className = $"{entityName}{ruleName}Rule";

        return $$"""
        using System.Threading;
        using System.Threading.Tasks;
        using Commerce.Core.Abstractions;
        using Commerce.Domain.Entities;

        namespace Commerce.Application.Features.{{entityName}}s.Rules;

        /// <summary>
        /// {{entityName}} varlığı için {{hookType}} aşamasında çalışan izole iş kuralı.
        /// </summary>
        public sealed class {{className}} : {{interfaceName}}
        {
            // TODO: Gerekli e-ticaret servislerini (Örn: IProductCatalogService, IInventoryService) enjekte edin.
            public {{className}}()
            {
            }

            public async Task<OperationResult> {{methodName}}({{entityName}} entity, CancellationToken ct = default)
            {
                await Task.CompletedTask;

                // TODO: İş kuralı doğrulamasını buraya yazın.
                bool canProceed = true;

                if (!canProceed)
                {
                    return OperationResult.Failure("{{entityName}} için {{ruleName}} kuralı doğrulamadan geçemedi.");
                }

                return OperationResult.Success();
            }
        }
        """;
    }

    public static string GenerateRuleUnitTest(string entityName, string hookType, string ruleName)
    {
        var methodName = $"On{hookType}Async";
        var className = $"{entityName}{ruleName}Rule";

        return $$"""
        using System.Threading;
        using System.Threading.Tasks;
        using Commerce.Application.Features.{{entityName}}s.Rules;
        using Commerce.Domain.Entities;
        using Xunit;

        namespace Commerce.UnitTests.Features.{{entityName}}s.Rules;

        public sealed class {{className}}Tests
        {
            [Fact]
            public async Task {{methodName}}_Should_ReturnSuccess_When_RuleIsSatisfied()
            {
                // Arrange
                var sut = new {{className}}();
                var entity = new {{entityName}}();

                // Act
                var result = await sut.{{methodName}}(entity, CancellationToken.None);

                // Assert
                Assert.True(result.IsSuccess);
            }
        }
        """;
    }

    public static string GenerateEjectedHandler(string entityName, string operation)
    {
        return $$"""
        using System.Threading;
        using System.Threading.Tasks;
        using Commerce.Core.Abstractions;
        using Commerce.Domain.Entities;

        namespace Commerce.Application.Features.{{entityName}}s;

        public partial record {{operation}}{{entityName}}Command(int Id, string Name);

        /// <summary>
        /// Bu sınıf 'archgen eject' komutu ile Source Generator'dan fiziksel dosyaya çıkarılmıştır.
        /// Smart Override mekanizması sayesinde Generator artık bu operasyonu bellekte üretmeyecektir.
        /// </summary>
        public sealed partial class {{operation}}{{entityName}}CommandHandler
        {
            private readonly ICommerceDbContext _dbContext;

            public {{operation}}{{entityName}}CommandHandler(ICommerceDbContext dbContext)
            {
                _dbContext = dbContext;
            }

            public async Task<OperationResult> HandleAsync({{operation}}{{entityName}}Command command, CancellationToken ct = default)
            {
                // TODO: Özel {{operation}} iş akışınızı (Örn: Elasticsearch indeks güncellemesi, Outbox event fırlatma) buraya yazın.
                var entity = await _dbContext.Set<{{entityName}}>().FindAsync(new object[] { command.Id }, ct);
                if (entity is null)
                    return OperationResult.Failure("{{entityName}} kaydı bulunamadı.");

                await _dbContext.SaveChangesAsync(ct);
                return OperationResult.Success();
            }
        }
        """;
    }
}

```

### 3. `AddRuleCommandHandler.cs` ve `EjectCommandHandler.cs`

```csharp
using ArchGen.Cli.Templates;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace ArchGen.Cli.Commands;

public static class AddRuleCommandHandler
{
    public static async Task ExecuteAsync(string entity, string hook, string ruleName, string rootPath)
    {
        var className = $"{entity}{ruleName}Rule";
        
        var ruleDir = Path.Combine(rootPath, "src", "Commerce.Application", "Features", $"{entity}s", "Rules");
        Directory.CreateDirectory(ruleDir);
        var ruleFilePath = Path.Combine(ruleDir, $"{className}.cs");

        var testDir = Path.Combine(rootPath, "tests", "Commerce.UnitTests", "Features", $"{entity}s", "Rules");
        Directory.CreateDirectory(testDir);
        var testFilePath = Path.Combine(testDir, $"{className}Tests.cs");

        if (File.Exists(ruleFilePath))
        {
            Console.WriteLine($"[UYARI] {className}.cs zaten mevcut! İşlem iptal edildi.");
            return;
        }

        await File.WriteAllTextAsync(ruleFilePath, TemplateEngine.GenerateBusinessRule(entity, hook, ruleName));
        await File.WriteAllTextAsync(testFilePath, TemplateEngine.GenerateRuleUnitTest(entity, hook, ruleName));

        Console.WriteLine($"[BAŞARILI] İş kuralı oluşturuldu : {Path.GetRelativePath(rootPath, ruleFilePath)}");
        Console.WriteLine($"[BAŞARILI] Birim testi oluşturuldu: {Path.GetRelativePath(rootPath, testFilePath)}");
        Console.WriteLine($"[BİLGİ] DI Kaydı: services.AddScoped<I{hook}Rule<{entity}>, {className}>();");
    }
}

public static class EjectCommandHandler
{
    public static async Task ExecuteAsync(string entity, string operation, string rootPath)
    {
        var entityFilePath = Path.Combine(rootPath, "src", "Commerce.Domain", "Entities", $"{entity}.cs");
        if (!File.Exists(entityFilePath))
        {
            Console.WriteLine($"[HATA] Entity dosyası bulunamadı: {entityFilePath}");
            return;
        }

        // 1. Roslyn AST ile Entity üzerindeki Attribute'tan ilgili operasyonu çıkar
        var sourceCode = await File.ReadAllTextAsync(entityFilePath);
        var updatedSourceCode = ExcludeOperationFromEntityAttribute(sourceCode, operation);
        await File.WriteAllTextAsync(entityFilePath, updatedSourceCode);

        // 2. Fiziksel Handler sınıfını diske yaz
        var featureDir = Path.Combine(rootPath, "src", "Commerce.Application", "Features", $"{entity}s");
        Directory.CreateDirectory(featureDir);
        var handlerFilePath = Path.Combine(featureDir, $"{operation}{entity}CommandHandler.cs");

        await File.WriteAllTextAsync(handlerFilePath, TemplateEngine.GenerateEjectedHandler(entity, operation));

        Console.WriteLine($"[EJECT] {entity}.cs içindeki [GenerateCrudEndpoints] attribute'undan '{operation}' çıkarıldı.");
        Console.WriteLine($"[EJECT] Fiziksel Handler oluşturuldu: {Path.GetRelativePath(rootPath, handlerFilePath)}");
    }

    private static string ExcludeOperationFromEntityAttribute(string sourceCode, string operationToExclude)
    {
        var tree = CSharpSyntaxTree.ParseText(sourceCode);
        var root = tree.GetCompilationUnitRoot();

        var attribute = root.DescendantNodes()
            .OfType<AttributeSyntax>()
            .FirstOrDefault(a => a.Name.ToString().Contains("GenerateCrudEndpoints"));

        if (attribute is null) return sourceCode;

        var newArgSyntax = SyntaxFactory.ParseAttributeArgument(
            $"Operations = CrudOperations.All & ~CrudOperations.{operationToExclude}");

        AttributeSyntax updatedAttribute;

        if (attribute.ArgumentList is null)
        {
            updatedAttribute = attribute.WithArgumentList(
                SyntaxFactory.AttributeArgumentList(SyntaxFactory.SingletonSeparatedList(newArgSyntax)));
        }
        else
        {
            var existingOpsArg = attribute.ArgumentList.Arguments
                .FirstOrDefault(a => a.NameEquals?.Name.Identifier.Text == "Operations");

            if (existingOpsArg is not null)
            {
                var combinedExpr = $"{existingOpsArg.Expression} & ~CrudOperations.{operationToExclude}";
                var replacedArg = existingOpsArg.WithExpression(SyntaxFactory.ParseExpression(combinedExpr));
                var newArgs = attribute.ArgumentList.Arguments.Replace(existingOpsArg, replacedArg);
                updatedAttribute = attribute.WithArgumentList(SyntaxFactory.AttributeArgumentList(newArgs));
            }
            else
            {
                var newArgs = attribute.ArgumentList.Arguments.Add(newArgSyntax);
                updatedAttribute = attribute.WithArgumentList(SyntaxFactory.AttributeArgumentList(newArgs));
            }
        }

        return root.ReplaceNode(attribute, updatedAttribute).ToFullString();
    }
}

```

---

## E-Ticaret Senaryosunda Uçtan Uca Kullanım ve Test

### 1. Domain Entity (`Category.cs`)

```csharp
using Commerce.Core.Attributes;

namespace Commerce.Domain.Entities;

[GenerateCrudEndpoints(RoutePrefix = "/api/categories")]
public class Category
{
    public int Id { get; set; }
    public string Name { get; set; } = string.Empty;
    public string Slug { get; set; } = string.Empty;
}

```

### 2. Terminalden Kural Ekleme ve Oluşan Kural Sınıfı

Geliştirici kategori silinmeden önce aktif ürün kontrolü yapmak istediğinde:

```bash
$ archgen add-rule Category --on BeforeDelete --name ActiveProductsCheck

```

Üretilen `CategoryActiveProductsCheckRule.cs` içine sadece şu iş mantığını yazar:

```csharp
using Commerce.Core.Abstractions;
using Commerce.Domain.Entities;

namespace Commerce.Application.Features.Categorys.Rules;

public sealed class CategoryActiveProductsCheckRule(IProductCatalogService catalogService) 
    : IBeforeDeleteRule<Category>
{
    public async Task<OperationResult> OnBeforeDeleteAsync(Category entity, CancellationToken ct = default)
    {
        bool hasActiveProducts = await catalogService.HasActiveProductsInCategoryAsync(entity.Id, ct);
        
        return hasActiveProducts 
            ? OperationResult.Failure("Bu kategoriye bağlı aktif ürünler bulunduğu için kategori silinemez.") 
            : OperationResult.Success();
    }
}

```

### 3. Birim Testi (`CategoryActiveProductsCheckRuleTests.cs`)

```csharp
public sealed class CategoryDeleteTests
{
    [Fact]
    public async Task DeleteCategory_Should_Fail_When_Category_Has_Active_Products()
    {
        // Arrange: Sadece ürün katalog servisini taklit ediyoruz (Mock/Fake)
        var mockCatalog = new Mock<IProductCatalogService>();
        mockCatalog.Setup(x => x.HasActiveProductsInCategoryAsync(10, It.IsAny<CancellationToken>()))
                   .ReturnsAsync(true); // Aktif ürün var!

        var rule = new CategoryActiveProductsCheckRule(mockCatalog.Object);
        var fakeDb = new FakeCommerceDbContext(new Category { Id = 10, Name = "Elektronik" });

        // Source Generator'ın bellekte ürettiği Handler'ı doğrudan test ediyoruz:
        var handler = new DeleteCategoryCommandHandler(fakeDb, [rule]);

        // Act
        var result = await handler.HandleAsync(new DeleteCategoryCommand(10));

        // Assert
        Assert.False(result.IsSuccess);
        Assert.Equal("Bu kategoriye bağlı aktif ürünler bulunduğu için kategori silinemez.", result.ErrorMessage);
    }
}

```