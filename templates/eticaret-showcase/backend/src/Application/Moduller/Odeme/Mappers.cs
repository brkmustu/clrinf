using System.Collections.Generic;
using System.Linq;
using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Responses;

namespace EticaretApp.Application.Features.Odemeler;

public static class OdemeMappingExtensions
{
    public static Odeme ToEntity(this CreateOdemeCommand command) =>
        new()
        {
        };

    public static CreatedOdemeResponse ToCreatedResponse(this Odeme odeme) =>
        new()
        {
            Id = odeme.Id,
        };

    public static Odeme ToEntity(this UpdateOdemeCommand command) =>
        new()
        {
            Id = command.Id,
        };

    public static UpdatedOdemeResponse ToUpdatedResponse(this Odeme odeme) =>
        new()
        {
            Id = odeme.Id,
        };

    public static DeletedOdemeResponse ToDeletedResponse(this Odeme odeme) =>
        new() { Id = odeme.Id };

    public static GetByIdOdemeResponse ToGetByIdResponse(this Odeme odeme) =>
        new()
        {
            Id = odeme.Id,
        };

    public static GetListOdemeListItemDto ToListItemDto(this Odeme odeme) =>
        new()
        {
            Id = odeme.Id,
        };

    public static GetListResponse<GetListOdemeListItemDto> ToPaginateResponse(this IList<Odeme> items) =>
        new()
        {
            Items = items.Select(x => x.ToListItemDto()).ToList(),
            Count = items.Count,
            Index = 0,
            Size = items.Count,
            Pages = 1
        };
}
