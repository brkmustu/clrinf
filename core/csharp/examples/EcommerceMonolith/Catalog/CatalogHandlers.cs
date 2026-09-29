using ClrinfCS.Core;
using EcommerceMonolith.Common.Functional;
using EcommerceMonolith.Domain.Common.Exceptions;

namespace EcommerceMonolith.Catalog;

public sealed class CreateProductHandler(ICatalogRepository repository) : IRequestHandler<CreateProductCommand, Result<Product, DomainError>>
{
    public async ValueTask<Result<Product, DomainError>> HandleAsync(CreateProductCommand request, RequestContext context, CancellationToken cancellationToken)
    {
        var product = new Product(
            Id: Guid.NewGuid().ToString("N")[..8],
            TenantId: request.TenantId,
            Sku: request.Sku,
            Name: request.Name,
            Price: request.Price,
            Status: "Active"
        );
        var saved = await repository.SaveAsync(product);
        return Result<Product, DomainError>.Success(saved);
    }
}

public sealed class GetProductHandler(ICatalogRepository repository) : IRequestHandler<GetProductQuery, Result<Product, DomainError>>
{
    public async ValueTask<Result<Product, DomainError>> HandleAsync(GetProductQuery request, RequestContext context, CancellationToken cancellationToken)
    {
        var product = await repository.GetByIdAsync(request.TenantId, request.ProductId);
        if (product is null)
        {
            return Result<Product, DomainError>.Failure(new DomainError("Product.NotFound", $"Product '{request.ProductId}' was not found."));
        }
        return Result<Product, DomainError>.Success(product);
    }
}
