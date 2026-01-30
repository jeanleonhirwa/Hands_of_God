/**
 * MCP Client - gRPC client for communicating with MCP Core Server
 */

import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import * as path from 'path';
import { logger } from './utils/logger';

export class MCPClient {
  private address: string;
  private fileClient: any;
  private commandClient: any;
  private gitClient: any;
  private snapshotClient: any;
  private systemClient: any;

  constructor(address: string) {
    this.address = address;
    this.initClients();
  }

  private initClients() {
    const protoPath = path.join(__dirname, '../../protos');
    const options: protoLoader.Options = {
      keepCase: true,
      longs: String,
      enums: String,
      defaults: true,
      oneofs: true
    };

    // Load proto files
    const fileProto = this.loadProto(path.join(protoPath, 'file_service.proto'), options);
    const commandProto = this.loadProto(path.join(protoPath, 'command_service.proto'), options);
    const gitProto = this.loadProto(path.join(protoPath, 'git_service.proto'), options);
    const snapshotProto = this.loadProto(path.join(protoPath, 'snapshot_service.proto'), options);
    const systemProto = this.loadProto(path.join(protoPath, 'system_service.proto'), options);

    const credentials = grpc.credentials.createInsecure();

    // Create clients
    this.fileClient = new (fileProto as any).mcp.file.FileService(this.address, credentials);
    this.commandClient = new (commandProto as any).mcp.command.CommandService(this.address, credentials);
    this.gitClient = new (gitProto as any).mcp.git.GitService(this.address, credentials);
    this.snapshotClient = new (snapshotProto as any).mcp.snapshot.SnapshotService(this.address, credentials);
    this.systemClient = new (systemProto as any).mcp.system.SystemService(this.address, credentials);
  }

  private loadProto(protoPath: string, options: protoLoader.Options) {
    const packageDefinition = protoLoader.loadSync(protoPath, options);
    return grpc.loadPackageDefinition(packageDefinition);
  }

  /**
   * Execute a tool by name with the given arguments
   */
  async executeTool(name: string, args: Record<string, unknown>, dryRun: boolean = true): Promise<unknown> {
    logger.debug('Executing tool', { name, args, dryRun });

    switch (name) {
      case 'read_file':
        return this.readFile(args.path as string);
      case 'create_file':
        return this.createFile(args.path as string, args.content as string, dryRun);
      case 'list_dir':
        return this.listDir(args.path as string);
      case 'run_command':
        return this.runCommand(
          args.command as string,
          (args.args as string[]) || [],
          args.cwd as string,
          dryRun
        );
      case 'git_status':
        return this.gitStatus(args.repo_path as string);
      case 'git_commit':
        return this.gitCommit(
          args.repo_path as string,
          args.message as string,
          args.files as string[],
          dryRun
        );
      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  }

  // File operations
  async readFile(filePath: string): Promise<{ content: string; sha256: string; size: number }> {
    return new Promise((resolve, reject) => {
      this.fileClient.ReadFile({ path: filePath }, (err: Error, response: any) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }

  async createFile(filePath: string, content: string, dryRun: boolean = true): Promise<unknown> {
    if (dryRun) {
      return {
        dryRun: true,
        path: filePath,
        contentLength: content.length,
        predictedEffects: [`Will create/overwrite file: ${filePath}`]
      };
    }

    return new Promise((resolve, reject) => {
      this.fileClient.CreateFile({ path: filePath, content }, (err: Error, response: any) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }

  async listDir(dirPath: string): Promise<unknown> {
    return new Promise((resolve, reject) => {
      this.fileClient.ListDir({ path: dirPath }, (err: Error, response: any) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }

  // Command operations
  async runCommand(
    command: string,
    args: string[],
    cwd?: string,
    dryRun: boolean = true
  ): Promise<unknown> {
    return new Promise((resolve, reject) => {
      this.commandClient.Run(
        { command, args, cwd: cwd || '', dry_run: dryRun },
        (err: Error, response: any) => {
          if (err) reject(err);
          else resolve(response);
        }
      );
    });
  }

  // Git operations
  async gitStatus(repoPath: string): Promise<unknown> {
    return new Promise((resolve, reject) => {
      this.gitClient.Status({ repo_path: repoPath }, (err: Error, response: any) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }

  async gitCommit(
    repoPath: string,
    message: string,
    files: string[],
    dryRun: boolean = true
  ): Promise<unknown> {
    if (dryRun) {
      return {
        dryRun: true,
        repoPath,
        message,
        files,
        predictedEffects: [`Will commit ${files.length} files with message: "${message}"`]
      };
    }

    return new Promise((resolve, reject) => {
      this.gitClient.Commit(
        { repo_path: repoPath, message, files },
        (err: Error, response: any) => {
          if (err) reject(err);
          else resolve(response);
        }
      );
    });
  }

  // Snapshot operations
  async createSnapshot(paths: string[], label: string): Promise<unknown> {
    return new Promise((resolve, reject) => {
      this.snapshotClient.Create({ paths, label }, (err: Error, response: any) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }

  async restoreSnapshot(snapshotId: string, targetPaths?: string[]): Promise<unknown> {
    return new Promise((resolve, reject) => {
      this.snapshotClient.Restore(
        { snapshot_id: snapshotId, target_paths: targetPaths || [] },
        (err: Error, response: any) => {
          if (err) reject(err);
          else resolve(response);
        }
      );
    });
  }

  // System operations
  async getSystemInfo(): Promise<unknown> {
    return new Promise((resolve, reject) => {
      this.systemClient.GetSystemInfo({}, (err: Error, response: any) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }

  async getAuditLogs(limit: number = 100): Promise<unknown> {
    return new Promise((resolve, reject) => {
      this.systemClient.GetAuditLogs({ limit }, (err: Error, response: any) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }
}
