using System.Collections.Generic;
using System.Linq;
using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Responses;

namespace EticaretApp.Application.Features.Kargolar;

public static class KargoMappingExtensions
{
    public static Kargo ToEntity(this CreateKargoCommand command) =>
        new()
        {
        };

    public static CreatedKargoResponse ToCreatedResponse(this Kargo kargo) =>
        new()
        {
            Id = kargo.Id,
        };

    public static Kargo ToEntity(this UpdateKargoCommand command) =>
        new()
        {
            Id = command.Id,
        };

    public static UpdatedKargoResponse ToUpdatedResponse(this Kargo kargo) =>
        new()
        {
            Id = kargo.Id,
        };

    public static DeletedKargoResponse ToDeletedResponse(this Kargo kargo) =>
        new() { Id = kargo.Id };

    public static GetByIdKargoResponse ToGetByIdResponse(this Kargo kargo) =>
        new()
        {
            Id = kargo.Id,
        };

    public static GetListKargoListItemDto ToListItemDto(this Kargo kargo) =>
        new()
        {
            Id = kargo.Id,
        };

    public static GetListResponse<GetListKargoListItemDto> ToPaginateResponse(this IList<Kargo> items) =>
        new()
        {
            Items = items.Select(x => x.ToListItemDto()).ToList(),
            Count = items.Count,
            Index = 0,
            Size = items.Count,
            Pages = 1
        };
}
