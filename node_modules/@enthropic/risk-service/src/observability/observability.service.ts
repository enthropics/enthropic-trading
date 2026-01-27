import { Injectable, Logger } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { NodeSDK } from '@opentelemetry/sdk-node';
import { getNodeAutoInstrumentations } from '@opentelemetry/auto-instrumentations-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';
import { OTLPMetricExporter } from '@opentelemetry/exporter-metrics-otlp-grpc';
import { PeriodicExportingMetricReader } from '@opentelemetry/sdk-metrics';
import { Resource } from '@opentelemetry/resources';
import { SemanticResourceAttributes } from '@opentelemetry/semantic-conventions';

@Injectable()
export class ObservabilityService {
  private readonly logger = new Logger(ObservabilityService.name);
  private sdk: NodeSDK;

  constructor(private configService: ConfigService) {}

  async initialize(): Promise<void> {
    const otlpEndpoint = this.configService.get<string>(
      'OTEL_EXPORTER_OTLP_ENDPOINT',
      'http://localhost:4317'
    );
    const serviceName = this.configService.get<string>(
      'OTEL_SERVICE_NAME',
      'risk-service'
    );

    this.sdk = new NodeSDK({
      resource: new Resource({
        [SemanticResourceAttributes.SERVICE_NAME]: serviceName,
        [SemanticResourceAttributes.SERVICE_VERSION]: '1.0.0',
        [SemanticResourceAttributes.DEPLOYMENT_ENVIRONMENT]:
          this.configService.get<string>('NODE_ENV', 'development'),
      }),
      traceExporter: new OTLPTraceExporter({ url: otlpEndpoint }),
      metricReader: new PeriodicExportingMetricReader({
        exporter: new OTLPMetricExporter({ url: otlpEndpoint }),
        exportIntervalMillis: 15000,
      }),
      instrumentations: [getNodeAutoInstrumentations()],
    });

    await this.sdk.start();
    this.logger.log(`OpenTelemetry initialized for ${serviceName}`);
  }

  async shutdown(): Promise<void> {
    await this.sdk.shutdown();
    this.logger.log('OpenTelemetry shut down');
  }
}
