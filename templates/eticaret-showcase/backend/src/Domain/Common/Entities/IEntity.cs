using System;

namespace EticaretApp.Domain.Common.Entities;

public interface IEntity<TId>
{
    TId Id { get; set; }
    DateTime CreatedDate { get; set; }
    DateTime? UpdatedDate { get; set; }
    DateTime? DeletedDate { get; set; }
}
