import { Controller, Get, Post, Body, Query, Req, UseGuards } from '@nestjs/common';
import { Request } from 'express';
import { RiskService } from './risk.service';
import { JwtAuthGuard, PermissionsGuard, RequirePermissions } from '../auth/auth.guard';
import { AuthenticatedUser } from '../auth/auth.types';
import { OrderRiskCheck } from './risk.types';

@Controller('risk')
@UseGuards(JwtAuthGuard, PermissionsGuard)
export class RiskController {
  constructor(private riskService: RiskService) {}

  @Post('check')
  @RequirePermissions('orders:create')
  async checkOrderRisk(@Req() req: Request, @Body() order: OrderRiskCheck) {
    const user = req.user as AuthenticatedUser;
    return this.riskService.checkOrderRisk(user, order);
  }

  @Get('summary')
  @RequirePermissions('risk:read')
  async getAccountRiskSummary(@Req() req: Request, @Query('accountId') accountId?: string) {
    const user = req.user as AuthenticatedUser;
    return this.riskService.getAccountRiskSummary(user, accountId || user.accountId);
  }

  @Get('positions')
  @RequirePermissions('positions:read')
  async getPositions(@Req() req: Request, @Query('accountId') accountId?: string) {
    const user = req.user as AuthenticatedUser;
    return this.riskService.getPositions(user, accountId);
  }

  @Get('orders')
  @RequirePermissions('orders:read')
  async getOrders(
    @Req() req: Request,
    @Query('accountId') accountId?: string,
    @Query('status') status?: string,
  ) {
    const user = req.user as AuthenticatedUser;
    return this.riskService.getOrders(user, accountId, status);
  }
}
