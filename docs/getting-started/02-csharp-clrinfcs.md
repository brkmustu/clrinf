# C#

`clrinfcs` hem .NET'e özgü üretim araçlarını hem de yeniden kullanılabilir
çekirdek/adaptörleri barındırır. Kendi projenizde çekirdek ve adaptörleri
doğrudan bağımsız domain modellerinizle kullanabilirsiniz.

```bash
export CLRINF_CONFORMANCE_DIR="$PWD/tests/conformance/fixtures"
dotnet build clrinfcs/clrinf.slnx -c Release
dotnet test clrinfcs/tests/ClrInfTests/ClrInfTests.csproj -c Release
```

Core API'leri, dispatcher, SQLite adaptörü ve çalışan belge örnekleri
[C# deposunda](../../clrinfcs/README.md) belgelenmiştir. Public error/context
JSON biçimi CLR property adlarına değil ortak sözleşmeye uyar.

SQLite referansını kullanırken iş verisi, inbox ve outbox aynı transaction'a
katılmalıdır. İş veriniz başka veritabanındaysa aynı garantiyi otomatik olarak
elde etmezsiniz. Süreç içi dispatcher için mesaj aracısı gerekmez.

`ValueTask` kullanılması bütün çağrı zincirinin sıfır tahsisli olduğunun
kanıtı değildir. Performans sonuçları senaryo/runtime ve ölçümle birlikte
sunulmalıdır. .NET 10 burada preview değil hedef framework'tür.
