using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Rules;
using EticaretApp.Application.Common.Exceptions;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Moduller.Stoklar;

public static class StokBusinessMessages
{
    public const string SectionName = "Stok";
    public const string StokNotExists = "Stok bulunamadı.";
    public const string StokAlreadyExists = "Bu Stok zaten mevcut.";
}

public class StokBusinessRules : BaseBusinessRules
{
    private readonly IStokRepository _stokRepository;

    public StokBusinessRules(IStokRepository stokRepository)
    {
        _stokRepository = stokRepository;
    }

    public async Task StokShouldExistWhenSelected(Stok? stok)
    {
        if (stok == null)
            throw new BusinessException(StokBusinessMessages.StokNotExists);
    }

    public async Task StokIdShouldExistWhenSelected(int id, CancellationToken cancellationToken)
    {
        Stok? stok = await _stokRepository.GetAsync(
            predicate: x => x.Id.Equals(id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await StokShouldExistWhenSelected(stok);
    }
}
