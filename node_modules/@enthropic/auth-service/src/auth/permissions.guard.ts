import { Injectable, CanActivate, ExecutionContext, ForbiddenException } from '@nestjs/common';
import { Reflector } from '@nestjs/core';

@Injectable()
export class PermissionsGuard implements CanActivate {
  constructor(private reflector: Reflector) {}

  canActivate(context: ExecutionContext): boolean {
    const requiredPermissions = this.reflector.getAllAndOverride<string[]>('permissions', [
      context.getHandler(),
      context.getClass(),
    ]);

    if (!requiredPermissions || requiredPermissions.length === 0) {
      return true;
    }

    const { user } = context.switchToHttp().getRequest();
    
    if (!user || !user.permissions) {
      throw new ForbiddenException('Access denied');
    }

    // Admin has all permissions
    if (user.permissions.includes('admin:full')) {
      return true;
    }

    const hasPermission = requiredPermissions.some(permission => 
      user.permissions.includes(permission)
    );

    if (!hasPermission) {
      throw new ForbiddenException(`Required permissions: ${requiredPermissions.join(', ')}`);
    }

    return true;
  }
}
