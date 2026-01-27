import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { PrismaModule } from './prisma/prisma.module';
import { AuthModule } from './auth/auth.module';
import { RiskModule } from './risk/risk.module';
import { HealthModule } from './health/health.module';
import { ObservabilityModule } from './observability/observability.module';
import { ResilienceModule } from './resilience/resilience.module';
import configuration from './config/configuration';

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
      load: [configuration],
    }),
    PrismaModule,
    ObservabilityModule,
    ResilienceModule,
    AuthModule,
    RiskModule,
    HealthModule,
  ],
})
export class AppModule {}
