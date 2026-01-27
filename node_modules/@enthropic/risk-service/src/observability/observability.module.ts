import { Module, Global, OnModuleInit } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { ObservabilityService } from './observability.service';
import { MetricsService } from './metrics.service';

@Global()
@Module({
  providers: [ObservabilityService, MetricsService],
  exports: [ObservabilityService, MetricsService],
})
export class ObservabilityModule implements OnModuleInit {
  constructor(private observabilityService: ObservabilityService) {}

  async onModuleInit() {
    await this.observabilityService.initialize();
  }
}
