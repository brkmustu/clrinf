using System.Collections.Generic;
using System.Linq;
using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Responses;

namespace EticaretApp.Application.Features.Kuponlar;

public static class KuponMappingExtensions
{
    public static Kupon ToEntity(this CreateKuponCommand command) =>
        new()
        {
        };

    public static CreatedKuponResponse ToCreatedResponse(this Kupon kupon) =>
        new()
        {
            Id = kupon.Id,
        };

    public static Kupon ToEntity(this UpdateKuponCommand command) =>
        new()
        {
            Id = command.Id,
        };

    public static UpdatedKuponResponse ToUpdatedResponse(this Kupon kupon) =>
        new()
        {
            Id = kupon.Id,
        };

    public static DeletedKuponResponse ToDeletedResponse(this Kupon kupon) =>
        new() { Id = kupon.Id };

    public static GetByIdKuponResponse ToGetByIdResponse(this Kupon kupon) =>
        new()
        {
            Id = kupon.Id,
        };

    public static GetListKuponListItemDto ToListItemDto(this Kupon kupon) =>
        new()
        {
            Id = kupon.Id,
        };

    public static GetListResponse<GetListKuponListItemDto> ToPaginateResponse(this IList<Kupon> items) =>
        new()
        {
            Items = items.Select(x => x.ToListItemDto()).ToList(),
            Count = items.Count,
            Index = 0,
            Size = items.Count,
            Pages = 1
        };
}
