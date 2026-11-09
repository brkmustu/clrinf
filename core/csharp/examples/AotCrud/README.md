# AotCrud — compile-time CRUD example

`Category` and `Brand` are plain entities with `[GenerateCrudEndpoints]`.
The source generator in `src/ClrinfCS.Generators` emits, at compile time:

- `{Entity}Dto` and `Create/Update/GetById/GetList/Delete` requests
- one handler per operation (using `ICrudStore<TEntity,TKey>` and `CrudRuleRunner`)
- one Minimal API endpoint per operation
- `AddGeneratedCrud()` and `MapGeneratedCrudEndpoints(...)`

Nothing except the entities and `ActiveProductsCheckRule` is on disk.

## Run

```bash
dotnet run --project core/csharp/examples/AotCrud
```

Then:

```bash
curl -H 'X-Tenant-Id: tenant-a' http://localhost:5000/api/categories
curl -H 'X-Tenant-Id: tenant-a' -H 'Content-Type: application/json' \
  -d '{"name":"Electronics","slug":"electronics"}' \
  http://localhost:5000/api/categories
curl -X DELETE -H 'X-Tenant-Id: tenant-a' http://localhost:5000/api/categories/1
```

Deleting category `id=1` returns `422` with an `ErrorEnvelope` because the demo
catalog reports active products; the hook is executed by `CrudRuleRunner`.

## Agent workflow

```bash
# Add a hook instead of editing a generated handler
clrinf rule new ActiveProductsCheck --lang csharp --entity Category --on before-delete

# Eject one operation to a physical file (Smart Override stops generating it)
clrinfcs eject Category --operation Update
```

Never read files under `obj/`; they are generated. A CRUD behaviour change is
either a hook rule or an eject.

## EF Core

Swap the in-memory stores for the tested adapter:

```csharp
services.AddScoped<ICrudStore<Category, int>>(
    sp => new EfCrudStore<Category, int, AppDbContext>(sp.GetRequiredService<AppDbContext>()));
```

`EfCrudStore` scopes reads/writes by `RequestContext.TenantId` for
`ITenantScoped` entities and maps optimistic-concurrency failures to HTTP 409.

## AOT

The default build is runnable (reflection-based JSON enabled). To try the Native
AOT publish path:

```bash
dotnet publish -r linux-x64 -p:EnableAotCrudPublish=true
```

The generator output has zero trim warnings on its own, but ASP.NET Core's
`RequestDelegateGenerator` cannot see delegates emitted by another source
generator, so `MapX(Delegate)` still reports `IL2026/IL3050` during AOT publish.
JSON serialization of generated DTOs has the same cross-generator visibility
limitation. See the open items in
`.context/aot_source_generator_core_implementation_plan.md` (Phase 4).
