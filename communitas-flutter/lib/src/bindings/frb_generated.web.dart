// This file is a stub for web builds.
// Web platform uses bridge mode (HTTP API) instead of FFI bindings.
// @generated

// ignore_for_file: unused_import, unused_element, unnecessary_import, duplicate_ignore, invalid_use_of_internal_member, annotate_overrides, non_constant_identifier_names, curly_braces_in_flow_control_structures, prefer_const_literals_to_create_immutables, unused_field

import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';
import 'flutter_api.dart';
import 'frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

/// Web platform implementation stub.
/// All methods throw UnsupportedError because web uses bridge mode.
abstract class CommunitasRustApiImplPlatform
    extends BaseApiImpl<CommunitasRustWire> {
  CommunitasRustApiImplPlatform({
    required super.handler,
    required super.wire,
    required super.generalizedFrbRustBinding,
    required super.portManager,
  });

  CrossPlatformFinalizerArg
      get rust_arc_decrement_strong_count_CommunitasApiPtr =>
          throw UnsupportedError('Web platform uses bridge mode');

  @protected
  CommunitasApi
      dco_decode_Auto_Owned_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
              dynamic raw) =>
          throw UnsupportedError('Web platform uses bridge mode');

  @protected
  CommunitasApi
      dco_decode_Auto_Ref_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
              dynamic raw) =>
          throw UnsupportedError('Web platform uses bridge mode');

  @protected
  CommunitasApi
      dco_decode_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
              dynamic raw) =>
          throw UnsupportedError('Web platform uses bridge mode');

  @protected
  String dco_decode_String(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  bool dco_decode_bool(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterSessionInfo dco_decode_box_autoadd_flutter_session_info(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int dco_decode_box_autoadd_u_16(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterEntity dco_decode_flutter_entity(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterEntityType dco_decode_flutter_entity_type(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterEvent dco_decode_flutter_event(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterNetworkInfo dco_decode_flutter_network_info(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterSessionInfo dco_decode_flutter_session_info(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterUserProfile dco_decode_flutter_user_profile(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterVaultInfo dco_decode_flutter_vault_info(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int dco_decode_i_32(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  PlatformInt64 dco_decode_i_64(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  List<FlutterEntity> dco_decode_list_flutter_entity(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  List<FlutterEvent> dco_decode_list_flutter_event(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  List<FlutterVaultInfo> dco_decode_list_flutter_vault_info(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  Uint8List dco_decode_list_prim_u_8_strict(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  String? dco_decode_opt_String(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterSessionInfo? dco_decode_opt_box_autoadd_flutter_session_info(
          dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int? dco_decode_opt_box_autoadd_u_16(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int dco_decode_u_16(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int dco_decode_u_32(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  BigInt dco_decode_u_64(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int dco_decode_u_8(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void dco_decode_unit(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  BigInt dco_decode_usize(dynamic raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  CommunitasApi
      sse_decode_Auto_Owned_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
              SseDeserializer deserializer) =>
          throw UnsupportedError('Web platform uses bridge mode');

  @protected
  CommunitasApi
      sse_decode_Auto_Ref_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
              SseDeserializer deserializer) =>
          throw UnsupportedError('Web platform uses bridge mode');

  @protected
  CommunitasApi
      sse_decode_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
              SseDeserializer deserializer) =>
          throw UnsupportedError('Web platform uses bridge mode');

  @protected
  String sse_decode_String(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  bool sse_decode_bool(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterSessionInfo sse_decode_box_autoadd_flutter_session_info(
          SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int sse_decode_box_autoadd_u_16(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterEntity sse_decode_flutter_entity(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterEntityType sse_decode_flutter_entity_type(
          SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterEvent sse_decode_flutter_event(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterNetworkInfo sse_decode_flutter_network_info(
          SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterSessionInfo sse_decode_flutter_session_info(
          SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterUserProfile sse_decode_flutter_user_profile(
          SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterVaultInfo sse_decode_flutter_vault_info(
          SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int sse_decode_i_32(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  PlatformInt64 sse_decode_i_64(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  List<FlutterEntity> sse_decode_list_flutter_entity(
          SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  List<FlutterEvent> sse_decode_list_flutter_event(
          SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  List<FlutterVaultInfo> sse_decode_list_flutter_vault_info(
          SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  Uint8List sse_decode_list_prim_u_8_strict(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  String? sse_decode_opt_String(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  FlutterSessionInfo? sse_decode_opt_box_autoadd_flutter_session_info(
          SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int? sse_decode_opt_box_autoadd_u_16(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int sse_decode_u_16(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int sse_decode_u_32(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  BigInt sse_decode_u_64(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int sse_decode_u_8(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_decode_unit(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  BigInt sse_decode_usize(SseDeserializer deserializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int cst_encode_Auto_Owned_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
          CommunitasApi raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int cst_encode_Auto_Ref_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
          CommunitasApi raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int cst_encode_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
          CommunitasApi raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  bool cst_encode_bool(bool raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int cst_encode_flutter_entity_type(FlutterEntityType raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int cst_encode_i_32(int raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int cst_encode_u_16(int raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int cst_encode_u_32(int raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  int cst_encode_u_8(int raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void cst_encode_unit(void raw) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_Auto_Owned_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
          CommunitasApi self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_Auto_Ref_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
          CommunitasApi self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerCommunitasApi(
          CommunitasApi self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_String(String self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_bool(bool self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_box_autoadd_flutter_session_info(
          FlutterSessionInfo self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_box_autoadd_u_16(int self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_flutter_entity(
          FlutterEntity self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_flutter_entity_type(
          FlutterEntityType self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_flutter_event(FlutterEvent self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_flutter_network_info(
          FlutterNetworkInfo self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_flutter_session_info(
          FlutterSessionInfo self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_flutter_user_profile(
          FlutterUserProfile self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_flutter_vault_info(
          FlutterVaultInfo self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_i_32(int self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_i_64(PlatformInt64 self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_list_flutter_entity(
          List<FlutterEntity> self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_list_flutter_event(
          List<FlutterEvent> self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_list_flutter_vault_info(
          List<FlutterVaultInfo> self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_list_prim_u_8_strict(
          Uint8List self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_opt_String(String? self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_opt_box_autoadd_flutter_session_info(
          FlutterSessionInfo? self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_opt_box_autoadd_u_16(int? self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_u_16(int self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_u_32(int self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_u_64(BigInt self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_u_8(int self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_unit(void self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');

  @protected
  void sse_encode_usize(BigInt self, SseSerializer serializer) =>
      throw UnsupportedError('Web platform uses bridge mode');
}

/// Web wire class stub.
/// Web platform uses HTTP bridge, not FFI bindings.
class CommunitasRustWire implements BaseWire {
  /// Factory constructor throws on web.
  factory CommunitasRustWire.fromExternalLibrary(ExternalLibrary lib) {
    throw UnsupportedError(
        'CommunitasRustWire is not available on web platform. Use bridge mode instead.');
  }

  CommunitasRustWire._();
}
