using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Rules;
using EticaretApp.Application.Common.Exceptions;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Moduller.Urunlar;

public static class UrunBusinessMessages
{
    public const string SectionName = "Urun";
    public const string UrunNotExists = "Urun bulunamadı.";
    public const string UrunAlreadyExists = "Bu Urun zaten mevcut.";
}

public class UrunBusinessRules : BaseBusinessRules
{
    private readonly IUrunRepository _urunRepository;

    public UrunBusinessRules(IUrunRepository urunRepository)
    {
        _urunRepository = urunRepository;
    }

    public async Task UrunShouldExistWhenSelected(Urun? urun)
    {
        if (urun == null)
            throw new BusinessException(UrunBusinessMessages.UrunNotExists);
    }

    public async Task UrunIdShouldExistWhenSelected(int id, CancellationToken cancellationToken)
    {
        Urun? urun = await _urunRepository.GetAsync(
            predicate: x => x.Id.Equals(id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await UrunShouldExistWhenSelected(urun);
    }
}
