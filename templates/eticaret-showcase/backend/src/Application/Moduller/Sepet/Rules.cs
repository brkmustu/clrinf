using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Rules;
using EticaretApp.Application.Common.Exceptions;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Moduller.Sepetler;

public static class SepetBusinessMessages
{
    public const string SectionName = "Sepet";
    public const string SepetNotExists = "Sepet bulunamadı.";
    public const string SepetAlreadyExists = "Bu Sepet zaten mevcut.";
}

public class SepetBusinessRules : BaseBusinessRules
{
    private readonly ISepetRepository _sepetRepository;

    public SepetBusinessRules(ISepetRepository sepetRepository)
    {
        _sepetRepository = sepetRepository;
    }

    public async Task SepetShouldExistWhenSelected(Sepet? sepet)
    {
        if (sepet == null)
            throw new BusinessException(SepetBusinessMessages.SepetNotExists);
    }

    public async Task SepetIdShouldExistWhenSelected(int id, CancellationToken cancellationToken)
    {
        Sepet? sepet = await _sepetRepository.GetAsync(
            predicate: x => x.Id.Equals(id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await SepetShouldExistWhenSelected(sepet);
    }
}
