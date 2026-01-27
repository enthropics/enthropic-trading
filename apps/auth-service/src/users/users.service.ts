import { Injectable, NotFoundException, ForbiddenException } from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';

@Injectable()
export class UsersService {
  constructor(private prisma: PrismaService) {}

  async findById(id: string) {
    const account = await this.prisma.account.findUnique({
      where: { id },
      include: {
        role: {
          include: {
            permissions: { include: { permission: true } },
          },
        },
      },
    });

    if (!account) {
      throw new NotFoundException('User not found');
    }

    return this.sanitizeUser(account);
  }

  async findAll(requesterId: string, requesterPermissions: string[]) {
    // Check if user has permission to view all accounts
    if (!requesterPermissions.includes('accounts:read_all') && !requesterPermissions.includes('admin:full')) {
      throw new ForbiddenException('Insufficient permissions');
    }

    const accounts = await this.prisma.account.findMany({
      include: {
        role: true,
      },
      orderBy: { createdAt: 'desc' },
    });

    return accounts.map(a => this.sanitizeUser(a));
  }

  async updateRole(userId: string, roleName: string, requesterPermissions: string[]) {
    if (!requesterPermissions.includes('admin:full')) {
      throw new ForbiddenException('Only admins can change roles');
    }

    const role = await this.prisma.role.findUnique({
      where: { name: roleName },
    });

    if (!role) {
      throw new NotFoundException('Role not found');
    }

    const updated = await this.prisma.account.update({
      where: { id: userId },
      data: { roleId: role.id },
      include: { role: true },
    });

    return this.sanitizeUser(updated);
  }

  async deactivate(userId: string, requesterPermissions: string[]) {
    if (!requesterPermissions.includes('admin:full')) {
      throw new ForbiddenException('Only admins can deactivate accounts');
    }

    const updated = await this.prisma.account.update({
      where: { id: userId },
      data: { isActive: false },
    });

    return { message: 'Account deactivated', userId: updated.id };
  }

  private sanitizeUser(account: any) {
    const { passwordHash, ...rest } = account;
    return {
      ...rest,
      permissions: account.role?.permissions?.map((rp: any) => rp.permission.name) || [],
    };
  }
}
