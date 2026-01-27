import { Controller, Get, Patch, Param, Body, UseGuards } from '@nestjs/common';
import { UsersService } from './users.service';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { PermissionsGuard } from '../auth/permissions.guard';
import { RequirePermissions } from '../auth/permissions.decorator';
import { CurrentUser } from '../auth/current-user.decorator';

@Controller('users')
@UseGuards(JwtAuthGuard, PermissionsGuard)
export class UsersController {
  constructor(private usersService: UsersService) {}

  @Get()
  @RequirePermissions('accounts:read_all')
  async findAll(@CurrentUser() user: any) {
    return this.usersService.findAll(user.id, user.permissions);
  }

  @Get(':id')
  @RequirePermissions('accounts:read')
  async findOne(@Param('id') id: string, @CurrentUser() user: any) {
    // Users can only view their own account unless they have read_all
    if (id !== user.id && !user.permissions.includes('accounts:read_all') && !user.permissions.includes('admin:full')) {
      return this.usersService.findById(user.id);
    }
    return this.usersService.findById(id);
  }

  @Patch(':id/role')
  @RequirePermissions('admin:full')
  async updateRole(
    @Param('id') id: string,
    @Body('role') role: string,
    @CurrentUser() user: any,
  ) {
    return this.usersService.updateRole(id, role, user.permissions);
  }

  @Patch(':id/deactivate')
  @RequirePermissions('admin:full')
  async deactivate(@Param('id') id: string, @CurrentUser() user: any) {
    return this.usersService.deactivate(id, user.permissions);
  }
}
