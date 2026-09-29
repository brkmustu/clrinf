# C#

`core/csharp` (`clrinfcs`) hem .NET'e özgü üretim araçlarını hem de yeniden kullanılabilir
çekirdek/adaptörleri barındırır. Kendi projenizde çekirdek ve adaptörleri
doğrudan bağımsız domain modellerinizle kullanabilirsiniz.

```bash
export CLRINF_CONFORMANCE_DIR="$PWD/tests/conformance/fixtures"
dotnet build core/csharp/clrinf.slnx -c Release
dotnet test core/csharp/tests/ClrinfCS.Tests/ClrinfCS.Tests.csproj -c Release
```

Core API'leri, dispatcher, SQLite adaptörü ve çalışan belge örnekleri
[C# çekirdeğinde](../../core/csharp/README.md) belgelenmiştir. Public error/context
JSON biçimi CLR property adlarına değil ortak sözleşmeye uyar.

SQLite referansını kullanırken iş verisi, inbox ve outbox aynı transaction'a
katılmalıdır. İş veriniz başka veritabanındaysa aynı garantiyi otomatik olarak
elde etmezsiniz. Süreç içi dispatcher için mesaj aracısı gerekmez.

`ValueTask` kullanılması bütün çağrı zincirinin sıfır tahsisli olduğunun
kanıtı değildir. Performans sonuçları senaryo/runtime ve ölçümle birlikte
sunulmalıdır. .NET 10 burada preview değil hedef framework'tür.
